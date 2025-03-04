use std::{
    mem::take,
    sync::{Arc, Weak},
    time::Duration,
};

use encoding_rs::{Encoding, UTF_8};
use getset::Getters;
use regex::Regex;
use tokio::{spawn, task::JoinHandle, time::interval};

use crate::core::entities::contact_status::ContactStatus;

pub trait BbsListenerDelegate {
    fn on_update_contact_status(&self, contact_status: &ContactStatus);
}

async fn fetch_html_title(url: &str) -> Option<String> {
    let res = reqwest::get(url).await.ok()?;
    let bytes = res.bytes().await.unwrap();
    let utf8 = String::from_utf8_lossy(&bytes).to_lowercase();
    let regex = r#"<meta charset="(.?*)">|<meta http-equiv="content-type" content="text/html; charset=(.?*)"(:? /)?>"#;
    let encoding = Regex::new(regex)
        .unwrap()
        .captures(utf8.as_ref())
        .and_then(|x| x.get(1).or_else(|| x.get(2)))
        .and_then(|x| Encoding::for_label(x.as_str().as_bytes()))
        .unwrap_or(UTF_8);
    let html = encoding.decode(&bytes).0;
    Some(
        Regex::new(r"<title>(.+?)</title>")
            .unwrap()
            .captures(html.as_ref())?[1]
            .to_owned(),
    )
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

fn listen_jpnkn_bbs_thread(
    url: &str,
    delegate: Weak<dyn Send + Sync + BbsListenerDelegate>,
) -> Option<(JoinHandle<()>, Arc<std::sync::Mutex<ContactStatus>>)> {
    let c = Regex::new(r"^https://bbs\.jpnkn\.com/test/read.cgi/([^/]+)/([0-9]+)/")
        .unwrap()
        .captures(url)?;
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
    Some((join_handle, contact_status))
}

#[derive(Getters)]
pub struct JpnknBbsListener {
    join_handle: JoinHandle<()>,
    contact_status: Arc<std::sync::Mutex<ContactStatus>>,
}

impl JpnknBbsListener {
    pub fn listen(
        url: &str,
        delegate: Weak<dyn Send + Sync + BbsListenerDelegate>,
    ) -> Option<Self> {
        if let Some(jpnkn_bbs_thread_listener) = listen_jpnkn_bbs_thread(url, delegate.clone()) {
            return Some(Self {
                join_handle: jpnkn_bbs_thread_listener.0,
                contact_status: jpnkn_bbs_thread_listener.1,
            });
        }
        None
    }

    pub fn contact_status(&self) -> ContactStatus {
        self.contact_status.lock().unwrap().clone()
    }

    pub fn abort(&mut self) {
        self.join_handle.abort();
    }
}

pub struct BbsListenerContainer {
    url: String,
    jpnkn_bbs_listener: Option<JpnknBbsListener>,
    title: Arc<std::sync::Mutex<String>>,
    delegate: Weak<dyn Send + Sync + BbsListenerDelegate>,
}

unsafe impl Send for BbsListenerContainer {}
unsafe impl Sync for BbsListenerContainer {}

impl BbsListenerContainer {
    pub fn new() -> Self {
        struct EmptyBbsListenerDelegate {}
        impl BbsListenerDelegate for EmptyBbsListenerDelegate {
            fn on_update_contact_status(&self, _contact_status: &ContactStatus) {}
        }

        let empty_delegate = Arc::new(EmptyBbsListenerDelegate {});
        let delegate = Arc::downgrade(&empty_delegate);
        Self {
            url: Default::default(),
            jpnkn_bbs_listener: None,
            title: Default::default(),
            delegate,
        }
    }

    pub fn contact_status(&self) -> ContactStatus {
        self.jpnkn_bbs_listener
            .as_ref()
            .map(|x| x.contact_status())
            .unwrap_or_else(|| ContactStatus {
                title: self.title.lock().unwrap().clone(),
                res_count: 0,
            })
    }

    pub fn set_delegate(&mut self, delegate: Weak<dyn Send + Sync + BbsListenerDelegate>) {
        self.delegate = delegate;
        if let Some(mut jpnkn_bbs_listener) = take(&mut self.jpnkn_bbs_listener) {
            jpnkn_bbs_listener.abort();
            self.jpnkn_bbs_listener = JpnknBbsListener::listen(&self.url, self.delegate.clone());
        }
    }

    pub fn set_url(&mut self, url: String) {
        if url == self.url {
            return;
        }
        self.url = url;
        *self.title.lock().unwrap() = Default::default();
        if let Some(current) = &mut self.jpnkn_bbs_listener {
            current.abort();
            self.jpnkn_bbs_listener = None;
        }
        self.delegate
            .upgrade()
            .unwrap()
            .on_update_contact_status(&Default::default());
        if let Some(jpnkn_bbs_thread_listener) =
            listen_jpnkn_bbs_thread(&self.url, self.delegate.clone())
        {
            self.jpnkn_bbs_listener = Some(JpnknBbsListener {
                join_handle: jpnkn_bbs_thread_listener.0,
                contact_status: jpnkn_bbs_thread_listener.1,
            });
        } else {
            let url = self.url.clone();
            let delegate = self.delegate.clone();
            let title_mutex = self.title.clone();
            spawn(async move {
                if let Some(title) = fetch_html_title(&url).await {
                    title_mutex.lock().unwrap().clone_from(&title);
                    delegate
                        .upgrade()
                        .unwrap()
                        .on_update_contact_status(&ContactStatus {
                            title,
                            res_count: 0,
                        });
                }
            });
        }
    }
}
