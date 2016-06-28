## Howto build packages

Get the packages sources (orig.tar and debian.tar) from [here](http://packages.ubuntu.com/yakkety/ffmpeg).

Extract the orig.tar. Then extract debian.tar into the previously extracted folder. So for example, you extract `ffmpeg` into `ffmpeg/` folder. Then you have to extract debian.tar into `ffmpeg/debian`.

Then, run `dpkg-buildpackage -us -uc` into `ffmpeg/`.

You're done!
