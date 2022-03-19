# Database Setup

## Postgres

Create a user and a database:

```
su - postgres
createuser -W lxpuser # enter password at prompt
createdb -O lxpuser lxpdb
```

config.yaml:

```yaml
database:
  enabled: true
  url: postgres://lxpuser:lxppass@localhost/lxpdb
```

## MySQL

```mysql
CREATE USER 'lxpuser'@'localhost' IDENTIFIED BY 'lxppass';
CREATE DATABASE lxp;
GRANT ALL PRIVILEGES ON lxpdb.* to lxpuser@localhost;
```

config.yaml:

```yaml
database:
  enabled: true
  url: mysql://lxpuser:lxppass@localhost/lxpdb
```


## SQLite

All you need to do is create an empty file, lxp-bridge will do the rest:

```
touch /some/path/to/lxp-database.db
```

config.yaml:

```yaml
database:
  enabled: true
  url: sqlite://some/path/to/lxp-database.db
```
