![image](https://github.com/leb-kuchen/cosmic-applet-places-status-indicator/assets/102472435/d1cb1542-8fdd-484c-b138-c44b2e24a933)


# Install
```sh
git clone https://github.com/leb-kuchen/cosmic-applet-places-status-indicator
cd cosmic-applet-places-status-indicator
cargo b -r
sudo just install
```
# Dependencies
(some may not be required)
```
Build-Depends:
  debhelper (>= 11),
  debhelper-compat (= 11),
  rustc ,
  cargo,
  libdbus-1-dev,
  libegl-dev,
  libpulse-dev,
  libudev-dev,
  libxkbcommon-dev,
  libwayland-dev,
  libinput-dev,
  just,
  pkg-config,
```
