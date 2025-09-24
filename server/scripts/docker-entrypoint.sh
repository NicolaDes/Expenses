#!/bin/sh
set -e

until pg_isready -h expenses-database -p 5432 -U postgres; do
  sleep 1
done

EXISTS=$(psql "$DATABASE_URL" -tAc "SELECT 1 FROM information_schema.tables WHERE table_name='accounts';")

if [ "$EXISTS" != "1" ]; then
  echo "⚡ Empty DB, launching migration..."
  /app/migration up
fi

echo "▶️ Starting server..."
exec /app/server
