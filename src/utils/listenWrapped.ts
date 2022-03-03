import { Event, EventCallback, EventName, listen } from '@tauri-apps/api/event';

const map = new Map<string, EventCallback<any>[]>();

// WTF: A original unlistenFn doesn't work.
export default async function listenWrapped<T>(
  event: EventName,
  handler: EventCallback<T>
) {
  if (!map.has(event)) {
    await listen(event, (ev: Event<T>) => {
      for (const item of map.get(event)!!) {
        item(ev);
      }
    });
    map.set(event, []);
  }

  map.get(event)!!.push(handler);
  return async () => {
    map.set(
      event,
      map.get(event)!!.filter((x) => x !== handler)
    );
  };
}
