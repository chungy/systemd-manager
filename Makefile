PREFIX?=/usr/local
target=$(DESTDIR)$(PREFIX)

all:
	cargo rustc --release -- -C opt-level=3 -C lto

install:
	install -d $(target)/bin/
	install -d $(target)/share/applications/
	install -d $(target)/share/polkit-1/actions/
	install -m 755 target/release/systemd-manager $(target)/bin/
	install -m 755 assets/systemd-manager-pkexec $(target)/bin/
	install -m 644 assets/systemd-manager.desktop $(target)/share/applications/
	install -m 644 assets/org.freedesktop.policykit.systemd-manager.policy $(target)/share/polkit-1/actions/

uninstall:
	rm $(target)/bin/systemd-manager
	rm $(target)/bin/systemd-manager-pkexec
	rm $(target)/share/applications/systemd-manager.desktop
	rm $(target)/share/polkit-1/actions/org.freedesktop.policykit.systemd-manager.policy

ubuntu:
	sudo apt install libgtk-3-dev
	cargo build --release
	strip target/release/systemd-manager
	sed "7s/.*/Architecture: $(shell dpkg --print-architecture)/g" -i debian/DEBIAN/control
	install -d debian/usr/bin
	install -d debian/usr/share/applications
	install -d debian/usr/share/polkit-1/actions/
	install -m 755 target/release/systemd-manager debian/usr/bin
	install -m 755 assets/systemd-manager-pkexec debian/usr/bin/
	install -m 644 assets/systemd-manager.desktop debian/usr/share/applications/
	install -m 644 assets/org.freedesktop.policykit.systemd-manager.policy debian/usr/share/polkit-1/actions
	fakeroot dpkg-deb --build debian systemd-manager.deb
	sudo dpkg -i systemd-manager.deb
