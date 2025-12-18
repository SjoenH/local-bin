#!/bin/bash

# Function to parse a SQL Server connection string
parse_connection_string() {
    local conn_string="$1"
    
    # 1. Clean up: Remove leading/trailing spaces and trailing semicolons
    local cleaned_string=$(echo "$conn_string" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//;s/;[[:space:]]*$//')

    # Associative array to hold the extracted values (requires Bash 4.0+)
    declare -A conn_params

    # 2. Iterate through semicolon-separated tokens
    IFS=';' read -ra tokens <<< "$cleaned_string"
    for token in "${tokens[@]}"; do
        if [[ $token =~ ^([^=]+)=(.*)$ ]]; then
            local key="${BASH_REMATCH[1]}"
            local value="${BASH_REMATCH[2]}"
            
            # Remove spaces from KEY and convert to uppercase for consistent storage
            local upper_key=$(echo "$key" | tr -d '[:space:]' | tr '[:lower:]' '[:upper:]')
            
            # Remove surrounding spaces from VALUE
            local trimmed_value=$(echo "$value" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')

            # Normalize common key names
            case "$upper_key" in
                DATASOURCE) upper_key="SERVER" ;;
                INITIALCATALOG) upper_key="DATABASE" ;;
                USERID) upper_key="USERID" ;;
            esac

            # Store in the associative array
            conn_params["$upper_key"]="$trimmed_value"
        fi
    done

    # --- Extracting and Normalizing Output ---

    local server_key="${conn_params[SERVER]}"
    local server=""
    local port="1433" 

    # Check for Hostname,Port format
    if [[ "$server_key" =~ ^([^,]+),([0-9]+)$ ]]; then
        server="${BASH_REMATCH[1]}"
        port="${BASH_REMATCH[2]}"
    elif [[ -n "$server_key" ]]; then
        server="$server_key"
    fi

    local database="${conn_params[DATABASE]}"
    local user="${conn_params[USERID]}"
    local password="${conn_params[PASSWORD]}"
    
    # Handle Integrated Security (Clears user/pass if Integrated Security is used)
    local integrated_security="${conn_params[INTEGRATEDSECURITY]}" 
    if [[ "$integrated_security" =~ ^(True|SSPI)$ ]]; then
        user=""
        password=""
    fi

    # --- Print Simplified Results ---
    echo "--- IntelliJ Connection Parameters ---"
    echo "Host: $server"
    echo "Port: $port"
    echo "Database: $database"
    echo "User: $user"
    echo "Password: $password"
    echo "--------------------------------------"
    
    # Optional: Print the full JDBC URL for the 'URL' field override
    if [[ -n "$server" ]]; then
        local jdbc_url="jdbc:sqlserver://${server}:${port};databaseName=${database}"
        
        # Add TrustServerCertificate if present (for self-signed certs)
        if [[ "${conn_params[TRUSTSERVERCERTIFICATE]}" =~ ^(True|Yes)$ ]]; then
            jdbc_url="${jdbc_url};trustServerCertificate=true"
        fi

        echo "JDBC URL (for override field): $jdbc_url"
    fi
}

# --- Main Script Execution ---

if [ -z "$1" ]; then
    echo "Usage: bash $0 \"<Your_Connection_String>\""
    echo ""
    echo "Example:"
    echo "bash $0 \"Data source=ServerName,60000;Initial Catalog=DBName;User ID=MyUser;Password=MySecret;TrustServerCertificate=True;\""
    exit 1
fi

USER_CONN_STRING="$1"
echo "Parsing Connection String:"
parse_connection_string "$USER_CONN_STRING"