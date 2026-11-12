# URL Tracker API

Rust API built with Axum and SQLx, backed by PostgreSQL.

## Docker deployment

Create a `.env` file with the deployment settings:

```dotenv
POSTGRES_USER=postgres
POSTGRES_PASSWORD=replace-with-a-strong-password
POSTGRES_DB=urlshort
ADMIN_TOKEN=replace-with-a-long-random-token
API_PORT=8000
API_QUERY_HOST=http://ip-api.com/json/
SENTRY_DSN=https://your-sentry-dsn
```

`SENTRY_DSN` is optional. When configured, IP geolocation lookup failures are reported to Sentry with a failure-reason tag.

Start the API and database:

```sh
docker compose up --build -d
```

The API is available at `http://localhost:8000`; `GET /healthz` is its health check. PostgreSQL data is kept in the `pg_data` Docker volume. Use `docker compose down` to stop the services.

All `/admin` routes require the `x-api-key` header set to `ADMIN_TOKEN`. Short URL creation accepts the JSON request used by the original API and query parameters for compatibility. Short URLs redirect from `/r/{code}`.

## Verify with Docker

Start the services, then check the API health endpoint:

```sh
docker compose up --build -d
curl --fail http://localhost:8000/healthz
```

Stop the services when finished with `docker compose down`.