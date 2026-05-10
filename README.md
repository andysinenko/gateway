# gateway
Just a learning project for exploring Rust and Axum. Designed for use with MyBooks.

## run service in Docker

There is docker-composer.yaml. It must be one layer up to folders with mybooks and gateway.
```
├── docker-compose.yml
├── mybooks/
│   ├── Dockerfile
│   └── src/
├── gateway/
│   ├── Dockerfile
│   └── src/
```

Run service: 
```$ docker compose up -d```

Run one service from docker-compose.yaml:

```$ docker compose up -d --build gateway```

Get logs from service in docker: 
```$ docker compose logs -f mybooks```

Get all logs in docker:
```$ docker compose logs -f```

Update one service in docker:
```$ docker compose up -d --build gateway```

Rebuild all services in docker:
```$ docker compose up -d --build```

### PostgreSQL tuning for MacOS for backend service myBooks

`vim /opt/homebrew/var/postgresql@16/postgresql.conf`

edit next parameters:
```
max_connections = 300 \
shared_buffers = 2GB \
work_mem = 64MB \
maintenance_work_mem = 512MB \
effective_cache_size = 4GB

random_page_cost = 1.1 \
wal_buffers = 16MB \
max_wal_size = 4GB \
min_wal_size = 1GB
```

Also:

```
$ sudo sysctl -w kern.sysv.shmmax=4294967296 \ 
$ sudo sysctl -w kern.sysv.shmall=1048576
```

### WRK test of gateway + mybooks

```
    $ wrk -t8 -c100 -d30s http://localhost:3000/api/books
```

### Test result
```
Running 1m test @ http://localhost:3000/api/books
8 threads and 200 connections
Thread Stats   Avg      Stdev     Max   +/- Stdev
Latency     1.43ms    4.07ms 291.23ms   99.84%
Req/Sec    17.87k     1.13k   23.24k    84.65%
Latency Distribution
50%    1.25ms
75%    1.61ms
90%    2.03ms
99%    3.13ms
8537115 requests in 1.00m, 97.83GB read
Requests/sec: 142239.72
Transfer/sec:      1.63GB
```

### Additional info
Installation of PostgreSQL:

``` $ brew install postgresql@16 ```

``` $ brew services start postgresql@16```

```$ createdb things_rust```

Initialize DB from sql script:

```
$ psql postgres -c "CREATE USER things_service WITH PASSWORD 'your_password';"
$ psql postgres -c "CREATE DATABASE things_rust OWNER things_service;"
$ psql postgres -c "GRANT ALL PRIVILEGES ON DATABASE things_rust TO things_service;"
$ psql things_rust -c "GRANT ALL ON SCHEMA things TO things_service;"
$ psql things_rust -c "ALTER ROLE things_service SET search_path TO things;"
$ psql things_rust < ./mybooks/compose/init.sql
```
Backup DB in Docker:
```docker exec things-postgres-1 pg_dump -U things_service things > backup.sql```

Restore backup DB in Docker:
```cat backup.sql | docker exec -i things-postgres-1 psql -U things_service -d things```

