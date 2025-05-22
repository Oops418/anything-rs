use crossbeam_channel::unbounded;
use indexify::{index_add, index_delete};
use notify::{
    Config, Error, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind,
};
use std::{path::Path, thread};
use tracing::{Level, debug, span, warn};

pub fn init_service() {
    thread::spawn(move || {
        let span = span!(Level::DEBUG, "sentry service thread");
        let _enter = span.enter();
        if let Err(e) = guard("/Users/kxyang/Personal/CodeSpaces/anything-rs/chinese") {
            warn!("guard error: {e:?}")
        }
    });
}

fn guard<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (event_sender, event_receiver) = unbounded::<Result<Event, Error>>();
    let mut watcher = RecommendedWatcher::new(event_sender, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in event_receiver {
        match res {
            Ok(event) => match event.kind {
                EventKind::Create(_) => {
                    for path in event.paths {
                        index_add(path.to_str().unwrap());
                        debug!("index add: {}", path.to_str().unwrap());
                    }
                }
                EventKind::Modify(kind) => {
                    if let ModifyKind::Name(_) = kind {
                        for path in event.paths {
                            let path_str = path.to_str().unwrap();
                            if Path::new(path.to_str().unwrap()).exists() {
                                index_add(path_str);
                                debug!("index modify: {}", path_str);
                            } else {
                                index_delete(path_str);
                                debug!("index delete: {}", path_str);
                            }
                        }
                    }
                }
                _ => {}
            },
            Err(error) => warn!("watch error: {:?}", error),
        }
    }

    Ok(())
}
