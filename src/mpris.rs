use crate::app::AppMessage;
use crate::*;
use crossbeam_channel::Sender;
use dbus::arg::{RefArg, Variant};
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::message::SignalArgs;
use dbus::strings::Path;
use dbus_tree::{Access, Factory};
use std::{collections::HashMap, rc::Rc, time::Duration};

type Metadata = HashMap<String, Variant<Box<dyn RefArg>>>;

pub fn run_dbus_server(mailbox: Arc<Sender<AppMessage>>) -> Rc<dbus::ffidisp::Connection> {
  let conn = Rc::new(
    dbus::ffidisp::Connection::get_private(dbus::ffidisp::BusType::Session)
      .expect("Failed to connect to dbus"),
  );
  conn
    .register_name(
      "org.mpris.MediaPlayer2.aquinas",
      dbus::ffidisp::NameFlag::ReplaceExisting as u32,
    )
    .expect("Failed to register dbus player name");

  let f = Factory::new_fn::<()>();

  let property_canquit = f
    .property::<bool, _>("CanQuit", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(false); // TODO
      Ok(())
    });

  let property_canraise = f
    .property::<bool, _>("CanRaise", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(false);
      Ok(())
    });

  let property_cansetfullscreen = f
    .property::<bool, _>("CanSetFullscreen", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(false);
      Ok(())
    });

  let property_hastracklist = f
    .property::<bool, _>("HasTrackList", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(false); // TODO
      Ok(())
    });

  let property_identity = f
    .property::<String, _>("Identity", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append("aquinas".to_string());
      Ok(())
    });

  let property_urischemes = f
    .property::<Vec<String>, _>("SupportedUriSchemes", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(Vec::new() as Vec<String>);
      Ok(())
    });

  let property_mimetypes = f
    .property::<Vec<String>, _>("SupportedMimeTypes", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(Vec::new() as Vec<String>);
      Ok(())
    });

  // https://specifications.freedesktop.org/mpris-spec/latest/Media_Player.html
  let interface = f
    .interface("org.mpris.MediaPlayer2", ())
    .add_p(property_canquit)
    .add_p(property_canraise)
    .add_p(property_cansetfullscreen)
    .add_p(property_hastracklist)
    .add_p(property_identity)
    .add_p(property_urischemes)
    .add_p(property_mimetypes);

  let property_playbackstatus = {
    f.property::<String, _>("PlaybackStatus", ())
      .access(Access::Read)
      .on_get(move |iter, _| {
        iter.append("Stopped".to_owned());
        Ok(())
      })
  };

  let property_loopstatus = {
    f.property::<String, _>("LoopStatus", ())
      .access(Access::ReadWrite)
      .on_get(move |iter, _| {
        iter.append("None".to_string());
        Ok(())
      })
      .on_set(move |_iter, _| Ok(()))
  };

  let property_metadata = {
    f.property::<HashMap<String, Variant<Box<dyn RefArg>>>, _>("Metadata", ())
      .access(Access::Read)
      .on_get(move |iter, _| {
        let metadata: Metadata = HashMap::new();
        iter.append(metadata);
        Ok(())
      })
  };

  let property_position = {
    f.property::<i64, _>("Position", ())
      .access(Access::Read)
      .on_get(move |iter, _| {
        iter.append(0 as i64);
        Ok(())
      })
  };

  let property_volume = {
    f.property::<f64, _>("Volume", ())
      .access(Access::ReadWrite)
      .on_get(move |i, _| {
        i.append(0.0);
        Ok(())
      })
      .on_set(move |i, _| Ok(()))
  };

  let property_rate = f
    .property::<f64, _>("Rate", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(1.0);
      Ok(())
    });

  let property_minrate = f
    .property::<f64, _>("MinimumRate", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(1.0);
      Ok(())
    });

  let property_maxrate = f
    .property::<f64, _>("MaximumRate", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(1.0);
      Ok(())
    });

  let property_canplay = f
    .property::<bool, _>("CanPlay", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(true);
      Ok(())
    });

  let property_canpause = f
    .property::<bool, _>("CanPause", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(true);
      Ok(())
    });

  let property_canseek = f
    .property::<bool, _>("CanSeek", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(true);
      Ok(())
    });

  let property_cancontrol = f
    .property::<bool, _>("CanControl", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(true);
      Ok(())
    });

  let property_cangonext = f
    .property::<bool, _>("CanGoNext", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(true);
      Ok(())
    });

  let property_cangoprevious = f
    .property::<bool, _>("CanGoPrevious", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(true);
      Ok(())
    });

  let property_shuffle = {
    f.property::<bool, _>("Shuffle", ())
      .access(Access::ReadWrite)
      .on_get(move |iter, _| {
        iter.append(false);
        Ok(())
      })
      .on_set(move |iter, _| Ok(()))
  };

  let property_cangoforward = f
    .property::<bool, _>("CanGoForward", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(true);
      Ok(())
    });

  let property_canrewind = f
    .property::<bool, _>("CanRewind", ())
    .access(Access::Read)
    .on_get(|iter, _| {
      iter.append(true);
      Ok(())
    });

  let method_playpause = {
    let mailbox = mailbox.clone();
    f.method("PlayPause", (), move |m| {
      let _ = mailbox.send(AppMessage::PlayPause);
      Ok(vec![m.msg.method_return()])
    })
  };

  let method_play = {
    let mailbox = mailbox.clone();
    f.method("Play", (), move |m| {
      let _ = mailbox.send(AppMessage::Play(None));
      Ok(vec![m.msg.method_return()])
    })
  };

  let method_pause = {
    let mailbox = mailbox.clone();
    f.method("Pause", (), move |m| {
      let _ = mailbox.send(AppMessage::Pause);
      Ok(vec![m.msg.method_return()])
    })
  };

  let method_stop = {
    let mailbox = mailbox.clone();
    f.method("Stop", (), move |m| {
      let _ = mailbox.send(AppMessage::Pause);
      Ok(vec![m.msg.method_return()])
    })
  };

  let method_next = {
    let mailbox = mailbox.clone();
    f.method("Next", (), move |m| {
      let _ = mailbox.send(AppMessage::Next);
      Ok(vec![m.msg.method_return()])
    })
  };

  let method_previous = {
    let mailbox = mailbox.clone();
    f.method("Previous", (), move |m| {
      let _ = mailbox.send(AppMessage::Prev);
      Ok(vec![m.msg.method_return()])
    })
  };

  let method_forward = { f.method("Forward", (), move |m| Ok(vec![m.msg.method_return()])) };
  let method_rewind = { f.method("Rewind", (), move |m| Ok(vec![m.msg.method_return()])) };
  let method_seek = { f.method("Seek", (), move |m| Ok(vec![m.msg.method_return()])) };
  let method_set_position =
    { f.method("SetPosition", (), move |m| Ok(vec![m.msg.method_return()])) };
  let method_openuri = { f.method("OpenUri", (), move |m| Ok(vec![m.msg.method_return()])) };

  // https://specifications.freedesktop.org/mpris-spec/latest/Player_Interface.html
  let interface_player = f
    .interface("org.mpris.MediaPlayer2.Player", ())
    .add_p(property_playbackstatus)
    .add_p(property_loopstatus)
    .add_p(property_metadata)
    .add_p(property_position)
    .add_p(property_volume)
    .add_p(property_rate)
    .add_p(property_minrate)
    .add_p(property_maxrate)
    .add_p(property_canplay)
    .add_p(property_canpause)
    .add_p(property_canseek)
    .add_p(property_cancontrol)
    .add_p(property_cangonext)
    .add_p(property_cangoprevious)
    .add_p(property_shuffle)
    .add_p(property_cangoforward)
    .add_p(property_canrewind)
    .add_m(method_playpause)
    .add_m(method_play)
    .add_m(method_pause)
    .add_m(method_stop)
    .add_m(method_next)
    .add_m(method_previous)
    .add_m(method_forward)
    .add_m(method_rewind)
    .add_m(method_seek)
    .add_m(method_set_position)
    .add_m(method_openuri);

  let tree = f.tree(()).add(
    f.object_path("/org/mpris/MediaPlayer2", ())
      .introspectable()
      .add(interface)
      .add(interface_player),
  );

  tree
    .set_registered(&conn, true)
    .expect("failed to register tree");

  conn.add_handler(tree);

  let mut changed: PropertiesPropertiesChanged = Default::default();
  changed.interface_name = "org.mpris.MediaPlayer2.Player".to_string();
  changed.changed_properties.insert(
    String::from("PlaybackStatus"),
    Variant(Box::new("Playing".to_string())),
  );

  let metadata: HashMap<String, Variant<Box<dyn RefArg>>> = HashMap::new();
  changed
    .changed_properties
    .insert("Metadata".to_string(), Variant(Box::new(metadata)));

  conn
    .send(changed.to_emit_message(&Path::new("/org/mpris/MediaPlayer2".to_string()).unwrap()))
    .unwrap();

  // panic!("here?");

  loop {
    if let Some(m) = conn.incoming(200).next() {
      // warn!("Unhandled dbus message: {:?}", m);
    }

    // std::thread::sleep(Duration::from_secs(1));
  }

  conn
}
