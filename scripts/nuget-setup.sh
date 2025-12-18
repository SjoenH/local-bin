#!/bin/bash

# Stier
NUGET_DIR="$HOME/.nuget/NuGet"
NUGET_CONFIG="$NUGET_DIR/NuGet.Config"

# 🛑 SJEKK OM FIL EKSISTERER FØRST
if [ -f "$NUGET_CONFIG" ]; then
    echo "==============================================================="
    echo "⚠️  ADVARSEL: EKSISTERENDE KONFIGURASJON FUNNET"
    echo "==============================================================="
    echo "Filen eksisterer allerede her: $NUGET_CONFIG"
    echo ""
    echo "For å beskytte oppsettet ditt, avbryter scriptet nå uten å gjøre endringer."
    echo "Hvis du vil sette opp på nytt: Slett filen eller gi den nytt navn manuelt."
    echo ""
    exit 1
fi

echo "==============================================================="
echo "🚀 NUGET-OPPSETT FOR SPAREBANK 1 UTVIKLING"
echo "==============================================================="

GITHUB_USER=""
GITHUB_TOKEN=""

# 🤖 SJEKKER OM GH CLI ER TILGJENGELIG
if command -v gh &> /dev/null; then
    echo -e "\n🤖 GitHub CLI (gh) ble funnet!"
    echo "Vil du bruke denne til å hente brukernavn og token automatisk?"
    echo "(Dette kan kreve at du logger inn på nytt for å godkjenne 'read:packages')"
    
    echo -e "\n❓ Kjør automatisk oppsett med gh cli? (y/n)"
    read -n 1 -r < /dev/tty
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "\n🔄 Oppdaterer rettigheter (scopes) for å inkludere 'read:packages'..."
        # Oppdaterer scope. Dette åpner nettleser ved behov.
        gh auth refresh -h github.com -s read:packages
        
        if [ $? -eq 0 ]; then
            echo "✅ Rettigheter OK."
            GITHUB_USER=$(gh api user -q .login)
            GITHUB_TOKEN=$(gh auth token)
            echo "📥 Hentet brukernavn: $GITHUB_USER"
            echo "🔑 Hentet token automatisk."
        else
            echo "❌ Feilet med å hente token fra gh. Faller tilbake til manuell metode."
        fi
    fi
fi

# 📝 MANUELL INPUT (HVIS GH IKKE BLE BRUKT)
if [ -z "$GITHUB_TOKEN" ]; then
    echo -e "\n1️⃣  GENERER TILGANGSTOKEN (PAT)"
    echo "Du trenger et 'Classic Token' med følgende oppsett:"
    echo "   - Navn: F.eks. 'NuGet Mac'"
    echo "   - Scopes: [X] read:packages (Viktig!)"
    echo "   - SSO: Husk å trykke 'Configure SSO' -> 'Authorize' for sparebank1utvikling"

    echo -e "\n❓ Vil du at jeg skal åpne GitHub-siden for deg i nettleseren? (y/n)"
    read -n 1 -r < /dev/tty
    echo "" 
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        open "https://github.com/settings/tokens"
    fi

    echo -e "\n---------------------------------------------------------------"
    echo "2️⃣  FYLL INN DETALJER"
    echo "Når du har generert og kopiert tokenet, lim det inn her:"

    read -p "👉 GitHub-brukernavn: " GITHUB_USER < /dev/tty
    read -sp "👉 GitHub Classic Token: " GITHUB_TOKEN < /dev/tty
    echo -e "\n"
fi

# Sjekk at vi faktisk fikk input
if [ -z "$GITHUB_USER" ] || [ -z "$GITHUB_TOKEN" ]; then
    echo "❌ Feil: Brukernavn eller token mangler. Avbryter."
    exit 1
fi

# Opprettelse og konfigurasjon
echo "⚙️  Oppretter NuGet.Config..."

mkdir -p "$NUGET_DIR"

cat <<EOF > "$NUGET_CONFIG"
<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <packageSources>
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" protocolVersion="3" />
    <add key="github" value="https://nuget.pkg.github.com/sparebank1utvikling/index.json" />
  </packageSources>

  <packageSourceCredentials>
    <github>
      <add key="Username" value="$GITHUB_USER" />
      <add key="ClearTextPassword" value="$GITHUB_TOKEN" />
    </github>
  </packageSourceCredentials>

  <packageSourceMapping>
    <packageSource key="nuget.org">
      <package pattern="*" />
    </packageSource>
    <packageSource key="github">
      <package pattern="KFK.*" />
    </packageSource>
  </packageSourceMapping>
</configuration>
EOF

echo -e "\n✅ Konfigurasjon lagret i: $NUGET_CONFIG"
echo "--- Registrerte kilder ---"
dotnet nuget list source

echo -e "\n🎉 Du er nå klar til å kjøre 'dotnet restore'!"