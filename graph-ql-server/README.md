# Graph-Ql-server

In this Project we create one graphql server. 


## Tools

I use the [GraphQl](https://graphql.org/). The GraphQl is a query language for API. If you want to read for GraphQl in Rust with axum Library touch [here](https://docs.rs/async-graphql-axum/latest/async_graphql_axum/) for the Documentation.</br>
I use the axum framework for my server. Read [here](https://docs.rs/axum/latest/axum/) for axum framework.</br>
My database is the MongoDB. Read [here](https://www.mongodb.com/developer/languages/rust/rust-mongodb-crud-tutorial/) for MongoDB with Rust.</br>
For my REST API, I use the [Postman](https://www.postman.com/product/what-is-postman/) API Platform.</br>


### Run Web Service

* Open Postman or [Playground](https://countries.trevorblades.com/) from your Broswer.
* Run from the command line `cargo run`.


### /api/graphql


* We have the entity `User`:
```
{
    "id": string,
    "name": string,
    "age": int,
    "languageId": string
}
```

* and We have the entity `Languages`:
```
{
    "id": string,
    "name": string
}
```

* Open Postman or Playground.
* Give the Post endpoint `/api/graphql`.
* After you can run your queries or mutations.

The Server connected in the `BoardingBase` from the Localhost with `MongoDB`.

For example:

* Take a user with `id: "10"`.
```
query{
    user(
        id: "10"
    )
    {
        id,
        name,
        age,

    }
}
```

* Take all users from the Base.
```
query{
    users{
        id,
        name,
        age
        languageId
    }
}
```

* Add one user in the base.
```
mutation{
    addUser(
        id: "12",
        name: "Maria",
        age: 40,
        languageId: "GR"
    ){
        id,
        name,
        age,
        languageId
    }
}
```

* Delete one user from the base.
```
mutation{
    deleteUser(
        id: "12",
    ){
        id,
        name,
        age,
        languageId
    }
}
```

* Add one user in the base.
```
mutation{
    updateUser(
        id: "12",
        name: "Maria Alexandrou",
    ){
        id,
        name,
        age,
        languageId
    }
}
```

* Take all Languages and users with this language.

```
query{
    languages{
        id,
        name,
        users{
            id,
            name,
            age
            languageId
        }
    }
}
```

