## Howto build packages

From this [website](http://www.deb-multimedia.org/dists/unstable/main/binary-amd64/), get the following packages:

 * libavcodec57_3.1-dmo1_amd64.deb
 * libavcodec-dev_3.1-dmo1_amd64.deb
 * libavformat57_3.1-dmo1_amd64.deb
 * libavformat-dev_3.1-dmo1_amd64.deb
 * libavutil55_3.1-dmo1_amd64.deb
 * libavutil-dev_3.1-dmo1_amd64.deb
 * libbluray1_0.9.3-dmo1_amd64.deb
 * libfaac0_1.28-dmo3_amd64.deb
 * libfdk-aac1_0.1.4-dmo1_amd64.deb
 * libilbc2_2.0.2-dmo4_amd64.deb
 * libkvazaar3_0.8.3-dmo2_amd64.deb
 * libmp3lame0_3.99.5-dmo5_amd64.deb
 * libswresample2_3.1-dmo1_amd64.deb
 * libswresample-dev_3.1-dmo1_amd64.deb
 * libx264-148_0.148.2705+git3f5ed56-dmo1_amd64.deb
 * libx265-79_1.9-dmo3_amd64.deb
 * libxvidcore4_1.3.4-dmo1_amd64.deb

Then run the following command for every packages:

```bash
dpkg-deb -x package_name ffmpeg-libs
```

Once done, you need to run the following commands:

```bash
rm -rf ffmpeg-libs/usr/share
dpkg-deb -e libavcodec57_3.1-dmo1_amd64.deb ffmpeg-libs/DEBIAN
```

Now edit `ffmpeg-libs/DEBIAN/control` to make it look like this:

```text
Package: ffmpeg-libs
Source: ffmpeg-dmo
Version: 10:3.1-dmo1
Architecture: amd64
Bugs: mailto:marillat@deb-multimedia.org
Maintainer: Christian Marillat <marillat@deb-multimedia.org>
Changed-By: Guillaume Gomez <guillaume1.gomez@gmail.com>
Installed-Size: 20203
Depends: pkg-config
Conflicts: libavcodec1-dev, libavcodec2-dev, libavcodeccvs-dev, libavcodeccvs51-dev (<= 3:20071129-0.0), libffmpeg0
Replaces: libavcodeccvs-dev, libavcodeccvs51-dev (<= 3:20071129-0.0), libxvidcore4, libxvidcore
Section: libdevel
Priority: optional
Multi-Arch: same
Homepage: http://ffmpeg.org/
Description: It contains all libraries needed by video-metadata-rs
```

Now just one last step:

```bash
dpkg-deb -b ffmpeg-libs
```

It'll generate a `ffmpeg-libs.deb` file that you can install using:

```bash
sudo dpkg -i ffmpeg-libs.deb
```

You're done!
