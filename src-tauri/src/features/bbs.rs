use std::{
    mem::take,
    sync::{Arc, Weak},
    time::Duration,
};

use getset::Getters;
use regex::Regex;
use tokio::{spawn, task::JoinHandle, time::interval};

use crate::core::entities::contact_status::ContactStatus;

pub trait BbsListenerDelegate {
    fn on_update_contact_status(&self, contact_status: &ContactStatus);
    fn on_update_contact_url(&self, url: String);
}

async fn fetch_subject_txt(board: &str) -> Vec<(u32, String, u16)> {
    let res = reqwest::get(format!("https://bbs.jpnkn.com/{}/subject.txt", board))
        .await
        .unwrap();
    let subject_txt = res.text().await.unwrap();
    subject_txt
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            log::trace!("{}", line);
            let c = Regex::new(r"^([0-9]+)\.dat<>(.+) \(([0-9]+)\)$")
                .unwrap()
                .captures(line)
                .unwrap();
            log::trace!("c {:?}", c);
            (
                c[1].parse::<u32>().unwrap(),
                c[2].to_owned(),
                c[3].parse::<u16>().unwrap(),
            )
        })
        .collect()
}

async fn tick(
    board: &str,
    thread: &mut u32,
    contact_status: &std::sync::Mutex<ContactStatus>,
    delegate: &Weak<dyn Send + Sync + BbsListenerDelegate>,
) {
    let list = fetch_subject_txt(board).await;
    let (_, item_title, item_res_count) = list
        .into_iter()
        .find(|(item_thread, _title, _res_count)| item_thread == thread)
        .unwrap();
    let mut contact_status = contact_status.lock().unwrap();
    if item_res_count <= contact_status.res_count {
        return;
    }
    contact_status.title = item_title;
    contact_status.res_count = item_res_count;
    delegate
        .upgrade()
        .unwrap()
        .on_update_contact_status(&contact_status);
}

#[derive(Getters)]
pub struct JpnknBbsListener {
    join_handle: JoinHandle<()>,
    #[getset(get = "pub")]
    url: String,
    contact_status: Arc<std::sync::Mutex<ContactStatus>>,
}

impl JpnknBbsListener {
    pub fn listen(
        url: String,
        delegate: Weak<dyn Send + Sync + BbsListenerDelegate>,
    ) -> Option<Self> {
        let c = Regex::new(r"^https://bbs\.jpnkn\.com/test/read.cgi/([^/]+)/([0-9]+)/")
            .unwrap()
            .captures(&url)?;
        log::trace!("{:?}", c);
        let board = c[1].to_owned();
        let mut thread = c[2].parse::<u32>().unwrap();
        let contact_status = Arc::new(std::sync::Mutex::new(Default::default()));
        let contact_status_clone = contact_status.clone();
        let join_handle = spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                tick(&board, &mut thread, &contact_status_clone, &delegate).await;
            }
        });
        Some(Self {
            join_handle,
            url,
            contact_status,
        })
    }

    pub fn contact_status(&self) -> ContactStatus {
        self.contact_status.lock().unwrap().clone()
    }

    pub fn abort(&mut self) {
        self.join_handle.abort();
    }
}

pub struct BbsListenerContainer {
    jpnkn_bbs_listener: Option<JpnknBbsListener>,
    delegate: Weak<dyn Send + Sync + BbsListenerDelegate>,
}

unsafe impl Send for BbsListenerContainer {}
unsafe impl Sync for BbsListenerContainer {}

impl BbsListenerContainer {
    pub fn new() -> Self {
        struct EmptyBbsListenerDelegate {}
        impl BbsListenerDelegate for EmptyBbsListenerDelegate {
            fn on_update_contact_status(&self, _contact_status: &ContactStatus) {}
            fn on_update_contact_url(&self, _url: String) {}
        }

        let empty_delegate = Arc::new(EmptyBbsListenerDelegate {});
        let delegate = Arc::downgrade(&empty_delegate);
        Self {
            jpnkn_bbs_listener: None,
            delegate,
        }
    }

    pub fn contact_status(&self) -> Option<ContactStatus> {
        Some(self.jpnkn_bbs_listener.as_ref()?.contact_status())
    }

    pub fn set_delegate(&mut self, delegate: Weak<dyn Send + Sync + BbsListenerDelegate>) {
        self.delegate = delegate.clone();
        if let Some(mut jpnkn_bbs_listener) = take(&mut self.jpnkn_bbs_listener) {
            jpnkn_bbs_listener.abort();
            self.jpnkn_bbs_listener =
                JpnknBbsListener::listen(jpnkn_bbs_listener.url, self.delegate.clone());
        }
    }

    pub fn set_url(&mut self, url: String) {
        if let Some(current) = &mut self.jpnkn_bbs_listener {
            if current.url() == &url {
                return;
            }
            current.abort();
        }
        self.delegate
            .upgrade()
            .unwrap()
            .on_update_contact_status(&Default::default());
        self.jpnkn_bbs_listener = JpnknBbsListener::listen(url, self.delegate.clone());
    }
}
