
# Installs files into the system
install: 
    sudo install -Dm0755 ./target/release/cosmic-applet-places-status-indicator  /usr/bin/cosmic-applet-places-status-indicator
    sudo install -Dm0644 data/dev.dominiccgeh.CosmicAppletPlacesStatusIndicator.desktop /usr/share/applications/dev.dominiccgeh.CosmicAppletPlacesStatusIndicator.desktop
    find 'data'/'icons' -type f -exec echo {} \; | rev | cut -d'/' -f-3 | rev | xargs -d '\n' -I {} sudo install -Dm0644 'data'/'icons'/{} /usr/share/icons/hicolor/{}

