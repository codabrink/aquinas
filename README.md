This is a little media player written for myself.

# Aquinas Media Player

A simple and straightforward terminal media player.

Built primarily with people who have a local and organized music collection in mind, with the philosophy that the folder heirarchy is enough.

## Controls

- **Enter** - Play
- **Space** - Play / Pause
- **Up / Down** - Go up / down file list
- **Left / Right** - Expand / contract highlighted folder
- **f** - Seek forward 2 seconds (**F** for 5 seconds)
- **b** - Seek backward 2 seconds (**B** for 5 seconds)
- **d** - Open change directory prompt
- **s** - Open search prompt (Search does nothing atm)

## Progress

State of the interface.

![image](https://user-images.githubusercontent.com/779390/146649058-0ae0e0bd-536b-4625-8884-0b84d4ff1d39.png)

### Features
- [x] Play music
- [x] File tree rendering
- [x] Change directories
- [x] Seek forward / backward
- [x] Gstreamer backend integration
- [x] Alternative audio backends (Gstreamer, Rodio)
  - [ ] Integrate [Symphonia](https://github.com/pdeljanov/Symphonia) backend
- [x] Automatically play next song
- [ ] Search
- [x] Sorting / ordering (Basic)
  - [ ] Advanced sorting / ordering
- [ ] Now playing info
- [ ] Help info
- [ ] General all around polish
- [ ] Song metadata
- [ ] Global media keys
