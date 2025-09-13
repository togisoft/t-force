#!/bin/bash

# Script to clear invalid auth tokens
# This can be used when a user has a valid JWT but the user doesn't exist in the database

echo "🔧 Clearing invalid auth token..."

# Check if we're in the right directory
if [ ! -f "docker-compose.yml" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Check if Docker containers are running
if ! docker ps | grep -q "tforce-backend"; then
    echo "❌ Error: Backend container is not running. Please start the application first."
    exit 1
fi

echo "✅ Backend container is running"

# Instructions for the user
echo ""
echo "📋 Instructions to fix the auth token issue:"
echo ""
echo "1. Open your browser's Developer Tools (F12)"
echo "2. Go to the Application/Storage tab"
echo "3. Find 'Cookies' in the left sidebar"
echo "4. Select your domain (localhost)"
echo "5. Find the 'auth_token' cookie"
echo "6. Delete the 'auth_token' cookie"
echo "7. Refresh the page"
echo "8. You will be redirected to the login page"
echo "9. Log in with the correct credentials:"
echo "   - Email: admin@example.com"
echo "   - Password: (check your environment variables or documentation)"
echo ""
echo "🔍 Alternative method using browser console:"
echo "   document.cookie = 'auth_token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';"
echo "   location.reload();"
echo ""

# Check if there are any users in the database
echo "📊 Current users in database:"
docker exec tforce-db psql -U postgres -d tforce -c "SELECT id, email, name, role FROM users;" 2>/dev/null || {
    echo "❌ Could not connect to database. Make sure the database container is running."
    exit 1
}

echo ""
echo "✅ Script completed. Follow the instructions above to clear your auth token." 