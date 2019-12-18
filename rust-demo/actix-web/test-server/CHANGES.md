# Changes

## [1.0.0] - 2019-12-13

### Changed

* Replaced `TestServer::start()` with `test_server()`


## [1.0.0-alpha.3] - 2019-12-07

### Changed

* Migrate to `std::future`


## [0.2.5] - 2019-09-17

### Changed

* Update serde_urlencoded to "0.6.1"
* Increase TestServerRuntime timeouts from 500ms to 3000ms

### Fixed

* Do not override current `System`


## [0.2.4] - 2019-07-18

* Update actix-server to 0.6

## [0.2.3] - 2019-07-16

* Add `delete`, `options`, `patch` methods to `TestServerRunner`

## [0.2.2] - 2019-06-16

* Add .put() and .sput() methods

## [0.2.1] - 2019-06-05

* Add license files

## [0.2.0] - 2019-05-12

* Update awc and actix-http deps

## [0.1.1] - 2019-04-24

* Always make new connection for http client


## [0.1.0] - 2019-04-16

* No changes


## [0.1.0-alpha.3] - 2019-04-02

* Request functions accept path #743


## [0.1.0-alpha.2] - 2019-03-29

* Added TestServerRuntime::load_body() method

* Update actix-http and awc libraries


## [0.1.0-alpha.1] - 2019-03-28

* Initial impl
