#!/bin/bash
set -e  # 🛑 Stopper scriptet hvis en kommando feiler (f.eks. hvis gh mangler)

CONFIG="$HOME/.nuget/NuGet/NuGet.Config"

# Helper: list sources and check GitHub NuGet source presence/Enabled
verify_github_source() {
    local SOURCES
    SOURCES=$(dotnet nuget list source 2>/dev/null || true)
    echo "$SOURCES"

    if echo "$SOURCES" | grep -q 'https://nuget.pkg.github.com/sparebank1utvikling/index.json'; then
        if echo "$SOURCES" | grep -B1 'https://nuget.pkg.github.com/sparebank1utvikling/index.json' | grep -q '\[Enabled\]'; then
            echo "✅ GitHub NuGet-kilden er registrert og Enabled: https://nuget.pkg.github.com/sparebank1utvikling/index.json"
            return 0
        else
            echo "⚠️ GitHub NuGet-kilden er registrert, men ser ikke ut til å være Enabled. Sjekk med 'dotnet nuget list source' og vurder å enable den."
            return 2
        fi
    else
        echo "⚠️ GitHub NuGet-kilden ble ikke funnet: https://nuget.pkg.github.com/sparebank1utvikling/index.json"
        return 1
    fi
}

# 1. Sjekk at vi ikke overskriver noe
if [ -f "$CONFIG" ]; then
    echo "⚠️  Filen $CONFIG finnes allerede... Innholdet er:"
    echo
    # Bruk helper for å vise kilder og gjøre kontrollen
    verify_github_source || true

    echo
    echo "Hvis du vil overskrive, fjern filen og kjør scriptet på nytt."
    exit 1
fi

echo "🚀 Setter opp NuGet..."

# 2. Sørg for at vi har riktig tilgang (read:packages)
# Hvis brukeren ikke er logget inn eller mangler scope, be om refresh/login.
if ! command -v gh >/dev/null 2>&1; then
    echo "⚠️ 'gh' CLI ikke funnet. Installer GitHub CLI for å fortsette."
    exit 1
fi

# Hvis gh allerede har 'read:packages', hopp over refresh
if gh auth status -h github.com 2>/dev/null | grep -q 'read:packages'; then
    echo "✅ GitHub CLI allerede autentisert med scope 'read:packages'."
else
    echo "🔒 Autentiserer mot GitHub for 'read:packages' (kan be om interaktiv innlogging)..."
    gh auth refresh -h github.com -s read:packages
fi

# 3. Opprett mappe og skriv fil
mkdir -p "$(dirname "$CONFIG")"

# Vi henter brukernavn og token direkte inne i skrive-operasjonen
cat <<EOF > "$CONFIG"
<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <packageSources>
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" protocolVersion="3" />
    <add key="github" value="https://nuget.pkg.github.com/sparebank1utvikling/index.json" />
  </packageSources>
  <packageSourceCredentials>
    <github>
      <add key="Username" value="$(gh api user -q .login)" />
      <add key="ClearTextPassword" value="$(gh auth token)" />
    </github>
  </packageSourceCredentials>
  <packageSourceMapping>
    <packageSource key="nuget.org"><package pattern="*" /></packageSource>
    <packageSource key="github"><package pattern="KFK.*" /></packageSource>
  </packageSourceMapping>
</configuration>
EOF

# Vis kilder og sjekk GitHub-kilden via helper
verify_github_source || true

echo "✅ Ferdig! Konfigurasjon lagret i $CONFIG."