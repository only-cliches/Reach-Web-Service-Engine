"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = (function (rws, db, fs, mail, elasticSearch) {
    rws.addCronJob("* * * * *", function () { return __awaiter(void 0, void 0, void 0, function () {
        return __generator(this, function (_a) {
            return [2 /*return*/];
        });
    }); });
    rws.addChannelListener("channel-name", function () { return __awaiter(void 0, void 0, void 0, function () {
        return __generator(this, function (_a) {
            return [2 /*return*/];
        });
    }); });
    rws.addComponent("g-form", {
        title: "Gravity Forms Form",
        description: "Display a gravity form",
        options: function () { return __awaiter(void 0, void 0, void 0, function () {
            var options;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, RWS.getOptions({})];
                    case 1:
                        options = _a.sent();
                        return [4 /*yield*/, RWS.saveOptions({})];
                    case 2:
                        _a.sent();
                        return [2 /*return*/, "<div></div>"];
                }
            });
        }); },
        render: function (args) { return __awaiter(void 0, void 0, void 0, function () {
            var options;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, RWS.getOptions()];
                    case 1:
                        options = _a.sent();
                        return [2 /*return*/, "<div>" + args.id + "</div>"];
                }
            });
        }); }
    });
    rws.addAction("route", function (headers, other, args) { return __awaiter(void 0, void 0, void 0, function () {
        return __generator(this, function (_a) {
            return [2 /*return*/];
        });
    }); }, 10);
    rws.addFilter("filter_name", function () { return __awaiter(void 0, void 0, void 0, function () {
        return __generator(this, function (_a) {
            return [2 /*return*/];
        });
    }); }, 10);
    // only works with DB native module
    db.migrate(1 /* current migration version */, function (migrate) { return __awaiter(void 0, void 0, void 0, function () {
        var users;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: 
                // make previous version of migration available
                // migrate.old_version = 0
                return [4 /*yield*/, migrate.newComp({
                        name: "login",
                        secure: false,
                        model: {
                            type: "table",
                            columns: [
                                ["username", { type: "string" }],
                                ["email", { type: "string" }],
                                ["phone", { type: "string" }]
                            ]
                        }
                    })];
                case 1:
                    // make previous version of migration available
                    // migrate.old_version = 0
                    _a.sent();
                    // user id index
                    return [4 /*yield*/, migrate.newIndex({
                            extend: "users",
                            name: "login-index",
                            watch: "login",
                            data_type: "string(32)",
                            value: function (insert, loginComponent) {
                                [
                                    loginComponent.username,
                                    loginComponent.email,
                                    loginComponent.phone
                                ].filter(function (v) { return v && v.length; }).forEach(function (v) {
                                    insert(String(v).toLowerCase());
                                });
                            }
                        })];
                case 2:
                    // user id index
                    _a.sent();
                    return [4 /*yield*/, migrate.newType({
                            name: "order-totals-by-day",
                            keyModel: { type: "u64" },
                            defaultComps: ["order-count"]
                        })];
                case 3:
                    _a.sent();
                    return [4 /*yield*/, migrate.newComp({
                            name: "order-count",
                            model: { type: "u64" },
                            secure: false
                        })];
                case 4:
                    _a.sent();
                    return [4 /*yield*/, migrate.newIndex({
                            extend: "order-totals-by-day",
                            name: "order-count-index",
                            watch: "order-count",
                            data_type: "u64",
                            value: function (index, orderCountComp) {
                                index.add(orderCountComp);
                            }
                        })];
                case 5:
                    _a.sent();
                    return [4 /*yield*/, migrate.newTrigger({
                            name: "user-search-index",
                            watch: "users.user-data",
                            call: function (userData) { return __awaiter(void 0, void 0, void 0, function () {
                                return __generator(this, function (_a) {
                                    switch (_a.label) {
                                        case 0: return [4 /*yield*/, elasticSearch.update(userData)];
                                        case 1:
                                            _a.sent();
                                            return [2 /*return*/];
                                    }
                                });
                            }); }
                        })];
                case 6:
                    _a.sent();
                    return [4 /*yield*/, migrate.newAggregate({
                            name: "calculate-daily-orders",
                            watch: "order.order-data",
                            apply: "order-totals-by-day.order-count",
                            key: function (orderData) {
                                return orderData.day;
                            },
                            value: function (isNewRecord, orderData, oldOrderCount) {
                                if (!oldOrderCount) {
                                    oldOrderCount = 0;
                                }
                                if (isNewRecord) {
                                    oldOrderCount++;
                                }
                                else {
                                    return false; // don't update anything
                                }
                                return oldOrderCount;
                            }
                        })];
                case 7:
                    _a.sent();
                    return [4 /*yield*/, migrate.newIndex({
                            extend: "blogs",
                            name: "authors",
                            watch: "blog-post-data",
                            data_type: "ulid",
                            value: function (index, blogPostData) {
                                index.add(blogPostData.authorID);
                            }
                        })];
                case 8:
                    _a.sent();
                    return [4 /*yield*/, db.select(["id", "or", "other", "components"]).from("users").all({ limit: 1 })];
                case 9:
                    users = _a.sent();
                    return [4 /*yield*/, db.select(["id", "or", "other", "components"]).from("users").find(["=", "entity ID"], { limit: 1 })];
                case 10:
                    users = _a.sent();
                    return [4 /*yield*/, db.select(["id", "or", "other", "components"]).from("users").find(["BETWEEN", ["lower ID", "higher ID"]], { limit: 1 })];
                case 11:
                    users = _a.sent();
                    return [4 /*yield*/, db.select(["id", "or"]).from("users.login-index").find(["=", "user@gmail.com"], { limit: 1 })];
                case 12:
                    // use index
                    users = _a.sent();
                    return [4 /*yield*/, db.select(["id", "or", "other", {
                                key: "blog-posts",
                                query: ["id", function (id_comp) { return __awaiter(void 0, void 0, void 0, function () {
                                        return __generator(this, function (_a) {
                                            switch (_a.label) {
                                                case 0: return [4 /*yield*/, db.select(["blog-post-data"]).from("blogs.authors").find(["=", id_comp], { limit: 5 })];
                                                case 1: return [2 /*return*/, _a.sent()];
                                            }
                                        });
                                    }); }]
                            }]).from("users").find(["BETWEEN", ["lower ID", "higher ID"]], { limit: 1 })];
                case 13:
                    // graph API
                    users = _a.sent();
                    /*
                    users = await db.select(["id", "or", "other", "components"]).fromAggr("post-count.users").find(["BETWEEN", ["lower ID", "higher ID"]], {limit: 1});
                    
                    // get users, blog-post[] => user
                    users = await db.select(["id", "or", "other", "components"]).fromRel("blog-author.users").find(["=", "entity blog-post ID"], {limit: 1});
                    // get blogs, blog-post[] <= user
                    users = await db.select(["id", "or", "other", "components"]).fromRel("blog-author.blog-posts").find(["=", "entity user ID"], {limit: 1});
            
                    // graph api
                    users = await db.select(["id", "or", "other", "components", async (id) => ({
                        name: "blog-posts",
                        query: await db.select(["blog-data"]).fromRel("blog-author.blog-posts").find(["=", id], {limit: 5})
                    })]).fromType("users").all({limit: 1});
                    */
                    return [4 /*yield*/, migrate.newComp({
                            name: "password",
                            model: { type: "string" },
                            secure: true
                        })];
                case 14:
                    /*
                    users = await db.select(["id", "or", "other", "components"]).fromAggr("post-count.users").find(["BETWEEN", ["lower ID", "higher ID"]], {limit: 1});
                    
                    // get users, blog-post[] => user
                    users = await db.select(["id", "or", "other", "components"]).fromRel("blog-author.users").find(["=", "entity blog-post ID"], {limit: 1});
                    // get blogs, blog-post[] <= user
                    users = await db.select(["id", "or", "other", "components"]).fromRel("blog-author.blog-posts").find(["=", "entity user ID"], {limit: 1});
            
                    // graph api
                    users = await db.select(["id", "or", "other", "components", async (id) => ({
                        name: "blog-posts",
                        query: await db.select(["blog-data"]).fromRel("blog-author.blog-posts").find(["=", id], {limit: 5})
                    })]).fromType("users").all({limit: 1});
                    */
                    _a.sent();
                    return [4 /*yield*/, migrate.newComp({
                            name: "forum-user-data",
                            model: { type: "table", columns: [
                                    ["favorite-post", { type: "ulid" }]
                                ] },
                            secure: true,
                            depends: ["login", "user-account"]
                        })];
                case 15:
                    _a.sent();
                    return [4 /*yield*/, migrate.newType({
                            name: "user",
                            keyModel: { type: "ulid" },
                            defaultComps: [
                                "user-account",
                                "login",
                                "password"
                            ]
                        })];
                case 16:
                    _a.sent();
                    return [4 /*yield*/, migrate.newType({
                            name: "blog-post",
                            keyModel: { type: "ulid" },
                            defaultComps: [
                                "blog-post-data"
                            ]
                        })];
                case 17:
                    _a.sent();
                    return [2 /*return*/];
            }
        });
    }); });
    // only works with FS native module
    fs.createFile( /* ... */);
    // only works with MAIL native module
    mail.createTemplate("template-name", "<h1>Head</h1>");
    /*cron_jobs: [
        {time: "* * * * *", call: async (RWS) => {

        }}
    ],
    channel_listeners: {
        "g-forms": () => {

        }
    },
    components: {
        "g-form": {
            title: "Gravity Forms Form",
            description: "Display a gravity form",
            options: async () => {
                const options = await RWS.getOptions({});
                await RWS.saveOptions({});
                return `<div></div>`;
            },
            render: async (args) => {
                const options = await RWS.getOptions();
                return `<div>${args.id}</div>`;
            }
        }
    },
    install: async () => {

    },
    uninstall: async () => {

    },
    inline: {
        rewrite: async () => {

        },
        head: async () => { // Tweak HTML head

        }
    },
    html_public: {
        rewrite: async () => {

        },
        head: async () => {

        },
        api: async () => {

        }
    }*/
});
/*
example.com

/ page-builder-app
/about-us about-us-app
/forum forum-app

404
500
403

sockets for ipc
one socket per module per deno (real) thread
https://crates.io/crates/parity-tokio-ipc

/about-us spa
/about-us/buy-our-shit spa

/forum mpa
/forum/page/1

/ page-builder-app
/this header-footer
    main-template
    [
        / page-builder-app
        /about-us about-us-app
        /forum [stock-ticker, forum-app, something-else]
    ]
*/ 
