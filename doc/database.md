# Database Setup

lxp-bridge can log input data to Postgres, MySQL, and SQLite. You just need to create an empty database and give lxp-bridge credentials to a user that can write to it. Tables will be created automatically at startup if they do not exist. Currently there's just one table; `inputs`.

Configuration is done with an `url` in the config file, see examples below. They are connection strings along the format of:

```
engine://[username[:password]@]host[:port][/dbname]
```

With the exception of SQLite which only needs a path to a file.

A new row will be added to the `inputs` table every time the inverter broadcasts data.

Note that the entry under `databases` is an array; you can configure multiple and lxp-bridge will store to each enabled one. Mixing and matching different database types is fine.

## Postgres

Create a user and a database:

```
su - postgres
createuser -W lxpuser # enter password at prompt
createdb -O lxpuser lxpdb
```

config.yaml:

```yaml
databases:
- enabled: true
  url: postgres://lxpuser:lxppass@localhost/lxpdb
```

## MySQL

Create the user and database in the MySQL console and grant it permissions:

```mysql
CREATE USER 'lxpuser'@'localhost' IDENTIFIED BY 'lxppass';
CREATE DATABASE lxpdb;
GRANT ALL PRIVILEGES ON lxpdb.* to lxpuser@localhost;
```

config.yaml:

```yaml
databases:
- enabled: true
  url: mysql://lxpuser:lxppass@localhost/lxpdb
```


## SQLite

All you need to do is create an empty file. lxp-bridge will not create the database file if it does not exist.

```
touch /some/path/to/lxp-database.db
```

config.yaml:

```yaml
databases:
- enabled: true
  url: sqlite://some/path/to/lxp-database.db
```
