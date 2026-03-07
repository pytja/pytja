#!/bin/bash
# Pytja Enterprise Release Bundler

RELEASE_NAME="pytja-enterprise-macos"
BUILD_DIR="target/release"

echo "--- Building Pytja Release Package ---"

# 1. Clean & Build
cargo build --release

# 2. Struktur erstellen
rm -rf $RELEASE_NAME
mkdir -p $RELEASE_NAME/config $RELEASE_NAME/certs $RELEASE_NAME/data/storage $RELEASE_NAME/logs

# 3. Binary kopieren
cp $BUILD_DIR/pytja $RELEASE_NAME/pytja-macos

# 4. Config kopieren (und umbenennen zu default.toml für den User)
cp config/release.toml $RELEASE_NAME/config/default.toml

# 5. Hilfsskripte erstellen
cat << 'EOF' > $RELEASE_NAME/setup_security.sh
#!/bin/bash
mkdir -p certs
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout certs/server.key -out certs/server.crt \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1,IP:::1"
echo "TLS-Zertifikate generiert."
EOF
chmod +x $RELEASE_NAME/setup_security.sh

# 6. README erstellen
cat << 'EOF' > $RELEASE_NAME/QUICKSTART.md
# Pytja Quickstart
1. Führe `./setup_security.sh` aus.
2. Registriere dich: `./pytja-macos registrar`
3. Starte Server: `./pytja-macos server`
4. Shell (neues Tab): `./pytja-macos shell`
EOF

# 7. Zippen
zip -r ${RELEASE_NAME}.zip $RELEASE_NAME
echo "--- Release bereit: ${RELEASE_NAME}.zip ---"