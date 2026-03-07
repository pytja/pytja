# Pytja Enterprise - Quickstart Guide

## Systemvoraussetzungen
Pytja benoetigt einen lokalen Redis-Cache fuer das hochperformante Session-Management.
macOS (via Homebrew):
brew install redis
brew services start redis

## Start der Plattform
1. Oeffnen Sie ein Terminal im entpackten Ordner.
2. Fuehren Sie die Applikation aus:
   ./pytja-macos
3. Der Setup-Wizard generiert automatisch Ihre TLS-Zertifikate,
   die lokale Datenbank und leitet Sie durch die Account-Erstellung.
