rm -vr dist
mkdir -v dist
cp -vr styles dist/styles
cp -v styles/terminal.rgs dist/style.rgs
cp -v target/release/rmulticlicker.exe target/release/rmulticlicker dist/
