## 9/29/20

  Added a basic config template for Domains.

## 9/28/20

  Better abstraction for the KV store to fix library coupling. Less type
  pollution.

  Made the db/table route now return actual JSON, db/table-bytes now returns raw
  byte arrays, which was the previous behavior.

  Completed put/delete and startup of domain servers, to use the DB and persist
  across restarts.

## 9/25/20

  Updated the admin server with static file service, an html page, and a proper
  db. Partially updated the admin init to use the db for loading and enabling
  domains.

## 9/17/20

  KV stores fully functional. Per-server DB files stored correctly now.

  Static files serving correctly.

  Default, fillable html file templates being served on all undefined routes

## 9/15/20

  Changing the static file server to use a single real root folder instead of
dynamic dispatch per-app.
