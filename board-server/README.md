# Rust Board server


## Tools

I use the axum framework for my server. Read [here](https://docs.rs/axum/latest/axum/) for axum framework.
My database is the MongoDB. Read [here](https://www.mongodb.com/developer/languages/rust/rust-mongodb-crud-tutorial/) for MongoDB with Rust.
For my REST API, I use the [Postman](https://www.postman.com/product/what-is-postman/) API Platform.


### Run Web Service

* Open Postman
* Run from the command line `cargo run`.


### /api/hello?name=x

* Open Postman
* Give the Get endpoint `/api/hello?name=x`, eg. where x = vaggelis, panagiotis, maria.
* The response is `Hello <x>`.

### /api/addData

* Open Postman.
* Give the Post endpoint `/api/addData` that requires a Content-Type: `application/json` with Body:
```
{
    "id": string,
    "name": string,
    "age": int
}
```
* After check the response and the data has been entered into the database.

### /api/getData/:id

* Open Postman.
* Give the Get endpoint `/api/getData/:id` for the search the `user` in `database`.
* The response is the Json file with `user` elements.
