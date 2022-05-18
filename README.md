# Aquinas Media Player

A simple and straightforward terminal media player.

Built primarily with people who have a local and organized music collection in mind, with the philosophy that the folder heirarchy is enough.

## Controls

| **Key** | **Action** |
| --- | ------ |
| **Enter** | Play |
| **Space** | Play / Pause |
| **Up** / **Down** | Navigate up / down file list |
| **Left** / **Right** | Expand / collapse highlighted folder |
| **f** | Seek forward 2 seconds (**F** for 5 seconds) |
| **b** | Seek backward 2 seconds (**B** for 5 seconds) |
| **d** | Open directory prompt (change folder) |
| **s** | Open search prompt |

## Progress

State of the interface.

![image](https://user-images.githubusercontent.com/779390/146649058-0ae0e0bd-536b-4625-8884-0b84d4ff1d39.png)

### Features
- [x] Play music
- [x] File tree rendering
- [x] Change directories
- [x] Seek forward / backward
- [x] [Symphonia](https://github.com/pdeljanov/Symphonia) backend integration
- [x] Gstreamer backend integration
- [x] Automatically play next song
- [x] Search
- [x] Sorting / ordering (Basic)
  - [ ] Advanced sorting / ordering
- [x] Global media keys (mpris)
- [ ] Help info
- [ ] Song metadata (disabled temporarily)



Installation
------------

Install RustUp:

    $ curl https://sh.rustup.rs -sSf | sh

Install Aquinas:

    $ cargo install aquinas
