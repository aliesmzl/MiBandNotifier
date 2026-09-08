export default function(global, globalThis, window, $app_exports$, $app_evaluate$) {
    var org_app_require = $app_require$;
    (function(global, globalThis, window, $app_exports$, $app_evaluate$) {
        var setTimeout = global.setTimeout;
        var setInterval = global.setInterval;
        var clearTimeout = global.clearTimeout;
        var clearInterval = global.clearInterval;
        var $app_require$1 = global.$app_require$ || org_app_require;
        var createPageHandler = function() {
            return (()=>{
                var __webpack_modules__ = {};
                var __webpack_module_cache__ = {};
                function __webpack_require__(moduleId) {
                    var cachedModule = __webpack_module_cache__[moduleId];
                    if (void 0 !== cachedModule) return cachedModule.exports;
                    var module = __webpack_module_cache__[moduleId] = {
                        exports: {}
                    };
                    __webpack_modules__[moduleId](module, module.exports, __webpack_require__);
                    return module.exports;
                }
                (()=>{
                    __webpack_require__.rv = ()=>"1.7.12";
                })();
                (()=>{
                    __webpack_require__.ruid = "bundler=rspack@1.7.12";
                })();
                var $app_style$ = [
                    [
                        [
                            [
                                0,
                                "page"
                            ]
                        ],
                        {
                            flexDirection: "column",
                            paddingTop: "12px",
                            paddingRight: "12px",
                            paddingBottom: "12px",
                            paddingLeft: "12px"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "header"
                            ]
                        ],
                        {
                            marginBottom: "10px"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "title"
                            ]
                        ],
                        {
                            fontSize: "20px",
                            color: "#ffffff",
                            fontWeight: "bold"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "form"
                            ]
                        ],
                        {
                            marginBottom: "12px"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "label"
                            ]
                        ],
                        {
                            fontSize: "14px",
                            color: "#b0bec5",
                            marginBottom: "6px"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "input"
                            ]
                        ],
                        {
                            height: "40px",
                            paddingTop: "8px",
                            paddingRight: "8px",
                            paddingBottom: "8px",
                            paddingLeft: "8px",
                            borderRadius: "8px",
                            backgroundColor: "#263238",
                            color: "#ffffff",
                            fontSize: "14px"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "hint"
                            ]
                        ],
                        {
                            marginTop: "6px"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "hint-text"
                            ]
                        ],
                        {
                            fontSize: "11px",
                            color: "#78909c"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "actions"
                            ]
                        ],
                        {
                            flexDirection: "row",
                            justifyContent: "center"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "button"
                            ]
                        ],
                        {
                            marginTop: "8px",
                            marginRight: "8px",
                            marginBottom: "8px",
                            marginLeft: "8px",
                            paddingTop: "10px",
                            paddingRight: "24px",
                            paddingBottom: "10px",
                            paddingLeft: "24px",
                            borderRadius: "10px",
                            backgroundColor: "#1976d2"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "button"
                            ],
                            [
                                0,
                                "secondary"
                            ]
                        ],
                        {
                            backgroundColor: "#455a64"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "button-text"
                            ]
                        ],
                        {
                            fontSize: "15px",
                            color: "#ffffff"
                        }
                    ]
                ];
                var $app_script$ = function __scriptModule__(module, exports, $app_require$1) {
                    "use strict";
                    Object.defineProperty(exports, "__esModule", {
                        value: true
                    });
                    exports.default = void 0;
                    var _system = _interopRequireDefault($app_require$1("@app-module/system.storage"));
                    function _interopRequireDefault(e) {
                        return e && e.__esModule ? e : {
                            default: e
                        };
                    }
                    var _default = exports.default = {
                        data: {
                            apiKey: ''
                        },
                        onInit () {
                            const self = this;
                            _system.default.get({
                                key: 'glm_api_key',
                                success: function(value) {
                                    if (value) self.apiKey = value;
                                }
                            });
                        },
                        onKeyChange (event) {
                            this.apiKey = event.value;
                        },
                        save () {
                            const self = this;
                            _system.default.set({
                                key: 'glm_api_key',
                                value: this.apiKey.trim(),
                                success: function() {
                                    self.$page.back();
                                }
                            });
                        },
                        back () {
                            this.$page.back();
                        }
                    };
                    const moduleOwn = exports.default || module.exports;
                    const accessors = [
                        'public',
                        'protected',
                        'private'
                    ];
                    if (moduleOwn.data && accessors.some(function(acc) {
                        return moduleOwn[acc];
                    })) throw new Error('页面VM对象中的属性data不可与"' + accessors.join(',') + '"同时存在，请使用private替换data名称');
                    if (!moduleOwn.data) {
                        moduleOwn.data = {};
                        moduleOwn._descriptor = {};
                        accessors.forEach(function(acc) {
                            const accType = typeof moduleOwn[acc];
                            if ('object' === accType) {
                                moduleOwn.data = Object.assign(moduleOwn.data, moduleOwn[acc]);
                                for(const name in moduleOwn[acc])moduleOwn._descriptor[name] = {
                                    access: acc
                                };
                            } else if ('function' === accType) console.warn('页面VM对象中的属性' + acc + '的值不能是函数，请使用对象');
                        });
                    }
                };
                var $app_template$ = function(vm) {
                    const _vm_ = vm || this;
                    return aiot.__ce__("div", {
                        __vm__: _vm_,
                        __opts__: {
                            classList: [
                                "page"
                            ]
                        }
                    }, [
                        aiot.__ce__("div", {
                            __vm__: _vm_,
                            __opts__: {
                                classList: [
                                    "header"
                                ]
                            }
                        }, [
                            aiot.__ce__("text", {
                                __vm__: _vm_,
                                __opts__: {
                                    classList: [
                                        "title"
                                    ],
                                    value: "设置"
                                }
                            }, [])
                        ]),
                        aiot.__ce__("div", {
                            __vm__: _vm_,
                            __opts__: {
                                classList: [
                                    "form"
                                ]
                            }
                        }, [
                            aiot.__ce__("text", {
                                __vm__: _vm_,
                                __opts__: {
                                    classList: [
                                        "label"
                                    ],
                                    value: "GLM API Key"
                                }
                            }, []),
                            aiot.__ce__("input", {
                                __vm__: _vm_,
                                __opts__: {
                                    classList: [
                                        "input"
                                    ],
                                    type: "text",
                                    value: function() {
                                        return _vm_.apiKey;
                                    },
                                    placeholder: "粘贴 bigmodel API Key",
                                    events: {
                                        change: function(evt) {
                                            return _vm_.onKeyChange(evt);
                                        }
                                    }
                                }
                            }, []),
                            aiot.__ce__("div", {
                                __vm__: _vm_,
                                __opts__: {
                                    classList: [
                                        "hint"
                                    ]
                                }
                            }, [
                                aiot.__ce__("text", {
                                    __vm__: _vm_,
                                    __opts__: {
                                        classList: [
                                            "hint-text"
                                        ],
                                        value: "bigmodel.cn 控制台 → API Keys 生成。Key 只保存在手环本地存储。"
                                    }
                                }, [])
                            ])
                        ]),
                        aiot.__ce__("div", {
                            __vm__: _vm_,
                            __opts__: {
                                classList: [
                                    "actions"
                                ]
                            }
                        }, [
                            aiot.__ce__("div", {
                                __vm__: _vm_,
                                __opts__: {
                                    classList: [
                                        "button"
                                    ],
                                    events: {
                                        click: function(evt) {
                                            return _vm_.save(evt);
                                        }
                                    }
                                }
                            }, [
                                aiot.__ce__("text", {
                                    __vm__: _vm_,
                                    __opts__: {
                                        classList: [
                                            "button-text"
                                        ],
                                        value: "保存"
                                    }
                                }, [])
                            ]),
                            aiot.__ce__("div", {
                                __vm__: _vm_,
                                __opts__: {
                                    classList: [
                                        "button",
                                        "secondary"
                                    ],
                                    events: {
                                        click: function(evt) {
                                            return _vm_.back(evt);
                                        }
                                    }
                                }
                            }, [
                                aiot.__ce__("text", {
                                    __vm__: _vm_,
                                    __opts__: {
                                        classList: [
                                            "button-text"
                                        ],
                                        value: "返回"
                                    }
                                }, [])
                            ])
                        ])
                    ]);
                };
                $app_exports$['entry'] = function($app_exports$) {
                    $app_script$({}, $app_exports$, $app_require$1);
                    $app_exports$.default.template = $app_template$;
                    $app_exports$.default.style = $app_style$;
                };
            })();
        };
        return createPageHandler();
    })(global, globalThis, window, $app_exports$, $app_evaluate$);
}
