use mongodb::bson::{self, doc};

pub fn installations() -> bson::Document {
    doc! {
        "wappierId" : "e6031bd6-17bf-542b-a202-82e6edfef394",
        "apiKey" : "cff9dc8b-1e74-461e-9f76-0337f77d77ed",
        "sessions" : {
            "367d950a-84b5-5a0a-bbf7-79bd06f164ba" : {
                "token" : "aa8033e3-6e11-4b50-a5b5-4c1a537301e7",
                "loyalty" : {
                    "audiences" : []
                },
                "pricing" : {
                    "audiences" : []
                },
                "campaign" : {
                    "audiences" : []
                },
                "generic" : {
                    "audiences" : []
                },
                "key" : "XX1sz7Iuoi1VS1wh",
                "header" : {
                    "clientId" : "93d4801d-d0f5-41a3-825f-908289395a87",
                    "clientSequence" : "1",
                    "apiKey" : "4840cabc-32aa-4727-bc27-555ff4e70c85",
                    "app" : "com.wappier-user-testing.shopify",
                    "platform" : "web",
                    "sdkVersion" : "3.4.0",
                    "clientSha" : "clientSha",
                    "timezone" : "Europe/Athens",
                    "unixTime" : "1638267060",
                    "appVersion" : "__appVersion__",
                    "deviceId" : "367d950a-84b5-5a0a-bbf7-79bd06f164ba",
                    "language" : "el-GR",
                    "sessionId" : "07cd6c7f-4b5f-4cca-95da-665d9b6ba356",
                    "username" : "loko",
                    "wappierId" : "e6031bd6-17bf-542b-a202-82e6edfef394"
                },
                "deviceIp" : "94.66.111.77",
                "build" : {
                    "osVersion" : "Android 6.0",
                    "brand" : "Chrome",
                    "model" : "96.0.4664.55",
                    "carrier" : "LG",
                    "lang" : "el-GR",
                    "country" : "GR"
                },
                "selectedLocale" : "en-US",
                "billingLocale" : "ZZ",
                "lastStringTable" : 1584033570465.0_f32,
                "version" : "4.15.0",
                "server" : "xanadu-qa",
                "createdAt" : "2021-11-30T10:11:00.624Z",
                "updatedAt" : "2021-11-30T10:11:16.548Z",
                "active" : false,
                "expiredAt" : "2121-11-30T10:11:00.624Z",
                "responseFormat" : "json",
                "updatedFrom" : "SDK",
                "createdFrom" : "SDK",
                "clientLocale" : "en-us"
            },
            "d799de6d-7925-5ab3-9581-8d0360a4a7d6" : {
                "active" : true,
                "billingLocale" : "ZZ",
                "build" : {
                    "brand" : "Chrome",
                    "carrier" : "LG",
                    "country" : "GR",
                    "lang" : "el-GR",
                    "model" : "96.0.4664.55",
                    "osVersion" : "Android 6.0"
                },
                "campaign" : {
                    "audiences" : []
                },
                "clientLocale" : "en-us",
                "createdAt" : "2021-11-30T10:11:23.711Z",
                "createdFrom" : "SDK",
                "deviceIp" : "94.66.111.77",
                "expiredAt" : "2121-11-30T10:11:23.711Z",
                "generic" : {
                    "audiences" : []
                },
                "header" : {
                    "apiKey" : "4840cabc-32aa-4727-bc27-555ff4e70c85",
                    "app" : "com.wappier-user-testing.shopify",
                    "appVersion" : "__appVersion__",
                    "clientId" : "dcc74d48-e5c6-434b-9ed7-30de6f5e6358",
                    "clientSequence" : "1",
                    "clientSha" : "clientSha",
                    "deviceId" : "d799de6d-7925-5ab3-9581-8d0360a4a7d6",
                    "language" : "el-GR",
                    "platform" : "web",
                    "sdkVersion" : "3.4.0",
                    "sessionId" : "70a3e55a-34ae-427a-aca2-5c4a2280c492",
                    "timezone" : "Europe/Athens",
                    "unixTime" : "1638267083",
                    "username" : "loko",
                    "wappierId" : "e6031bd6-17bf-542b-a202-82e6edfef394"
                },
                "key" : "n9dvzg4IKv9QViQg",
                "lastStringTable" : 1584033570465.0_f32,
                "loyalty" : {
                    "audiences" : []
                },
                "pricing" : {
                    "audiences" : []
                },
                "responseFormat" : "json",
                "selectedLocale" : "en-US",
                "server" : "xanadu-qa",
                "token" : "50955e89-eb35-4322-9b42-5e2711337994",
                "updatedAt" : "2021-11-30T10:11:39.175Z",
                "updatedFrom" : "SDK",
                "version" : "4.15.0"
            }
        }
    }
}
