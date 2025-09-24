DO
$$
BEGIN
   IF NOT EXISTS (
      SELECT FROM pg_database WHERE datname = 'expenses'
   ) THEN
      PERFORM dblink_exec('dbname=' || current_database(), 'CREATE DATABASE expenses');
   END IF;
END
$$;
