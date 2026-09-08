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
                    __webpack_require__.g = (()=>{
                        if ('object' == typeof globalThis) return globalThis;
                        try {
                            return this || new Function('return this')();
                        } catch (e) {
                            if ('object' == typeof window) return window;
                        }
                    })();
                })();
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
                            flexDirection: "row",
                            justifyContent: "space-between",
                            marginBottom: "8px"
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
                                "refresh"
                            ]
                        ],
                        {
                            fontSize: "14px",
                            color: "#64b5f6"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "card"
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
                                "row"
                            ]
                        ],
                        {
                            flexDirection: "row",
                            alignItems: "center",
                            marginBottom: "10px"
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
                            width: "70px",
                            fontSize: "14px",
                            color: "#b0bec5"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "bar-track"
                            ]
                        ],
                        {
                            width: "160px",
                            height: "10px",
                            borderRadius: "5px",
                            backgroundColor: "#37474f"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "bar-fill"
                            ]
                        ],
                        {
                            height: "10px",
                            borderRadius: "5px",
                            backgroundColor: "#4fc3f7"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "percent"
                            ]
                        ],
                        {
                            width: "44px",
                            fontSize: "14px",
                            color: "#ffffff",
                            marginLeft: "6px",
                            textAlign: "right"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "reset"
                            ]
                        ],
                        {
                            fontSize: "11px",
                            color: "#78909c",
                            marginLeft: "6px"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "error-text"
                            ]
                        ],
                        {
                            fontSize: "14px",
                            color: "#ef5350"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "loading"
                            ]
                        ],
                        {
                            fontSize: "14px",
                            color: "#b0bec5"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "footer"
                            ]
                        ],
                        {
                            marginTop: "4px",
                            flexDirection: "row",
                            justifyContent: "center"
                        }
                    ],
                    [
                        [
                            [
                                0,
                                "link"
                            ]
                        ],
                        {
                            fontSize: "13px",
                            color: "#64b5f6"
                        }
                    ]
                ];
                var $app_script$ = function __scriptModule__(module, exports, $app_require$1) {
                    "use strict";
                    Object.defineProperty(exports, "__esModule", {
                        value: true
                    });
                    exports.default = void 0;
                    var _system = _interopRequireDefault($app_require$1("@app-module/system.fetch"));
                    var _system2 = _interopRequireDefault($app_require$1("@app-module/system.storage"));
                    function _interopRequireDefault(e) {
                        return e && e.__esModule ? e : {
                            default: e
                        };
                    }
                    function _interopRequireWildcard(e, t) {
                        if ("function" == typeof WeakMap) var r = new WeakMap(), n = new WeakMap();
                        return (_interopRequireWildcard = function(e, t) {
                            if (!t && e && e.__esModule) return e;
                            var o, i, f = {
                                __proto__: null,
                                default: e
                            };
                            if (null === e || "object" != typeof e && "function" != typeof e) return f;
                            if (o = t ? n : r) {
                                if (o.has(e)) return o.get(e);
                                o.set(e, f);
                            }
                            for(const t in e)"default" !== t && ({}).hasOwnProperty.call(e, t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e, t)) && (i.get || i.set) ? o(f, t, i) : f[t] = e[t]);
                            return f;
                        })(e, t);
                    }
                    const ENDPOINT = 'https://open.bigmodel.cn/api/monitor/usage/quota/limit';
                    const WARN_THRESHOLD = 80;
                    var _default = exports.default = {
                        data: {
                            loaded: false,
                            error: '',
                            limits: []
                        },
                        onInit () {
                            this.refresh();
                        },
                        refresh () {
                            const self = this;
                            _system2.default.get({
                                key: 'glm_api_key',
                                success: function(value) {
                                    if (!value) {
                                        self.error = '未设置 API Key，请先到设置页';
                                        self.loaded = false;
                                        return;
                                    }
                                    self.query(value);
                                },
                                fail: function() {
                                    self.error = '未设置 API Key，请先到设置页';
                                    self.loaded = false;
                                }
                            });
                        },
                        query (apiKey) {
                            const self = this;
                            _system.default.fetch({
                                url: ENDPOINT,
                                method: 'GET',
                                header: {
                                    Authorization: 'Bearer ' + apiKey,
                                    Accept: 'application/json'
                                },
                                success: function(response) {
                                    try {
                                        const parsed = JSON.parse(response.data);
                                        if (!parsed.success || !parsed.data || !parsed.data.limits) {
                                            self.error = '接口返回异常 code=' + parsed.code;
                                            self.loaded = false;
                                            return;
                                        }
                                        self.renderLimits(parsed.data.limits);
                                    } catch (e) {
                                        self.error = '解析失败';
                                        self.loaded = false;
                                    }
                                },
                                fail: function(error, code) {
                                    self.error = '请求失败 code=' + code;
                                    self.loaded = false;
                                }
                            });
                        },
                        renderLimits (limits) {
                            const rows = [];
                            let maxPercent = 0;
                            for(let i = 0; i < limits.length; i++){
                                const item = limits[i];
                                if ('TOKENS_LIMIT' !== item.limitType && 'CREDIT_LIMIT' !== item.limitType) continue;
                                const label = this.windowLabel(item.unit, item.number);
                                if (!item.percentage && 0 !== item.percentage) continue;
                                const percent = Math.round(item.percentage + 0.5);
                                rows.push({
                                    label: label,
                                    percent: percent,
                                    width: Math.round(1.6 * percent),
                                    reset: item.nextResetTime ? this.formatReset(item.nextResetTime) : ''
                                });
                                if (percent > maxPercent) maxPercent = percent;
                            }
                            this.limits = rows;
                            this.error = '';
                            this.loaded = rows.length > 0;
                            if (0 === rows.length) {
                                this.error = '无窗口数据';
                                this.loaded = false;
                            }
                            if (maxPercent >= WARN_THRESHOLD) this.vibrate();
                        },
                        windowLabel (unit, number) {
                            if (3 === unit && 5 === number) return '5小时窗';
                            if (6 === unit && 1 === number) return '周窗';
                            if (5 === unit && 1 === number) return '月窗';
                            return '窗口';
                        },
                        formatReset (raw) {
                            let millis = raw;
                            if (raw < 10000000000) millis = 1000 * raw;
                            const date = new Date(millis);
                            const pad = function(value) {
                                return value < 10 ? '0' + value : '' + value;
                            };
                            return pad(date.getMonth() + 1) + '-' + pad(date.getDate()) + ' ' + pad(date.getHours()) + ':' + pad(date.getMinutes());
                        },
                        vibrate () {
                            Promise.resolve().then(()=>_interopRequireWildcard($app_require$1("@app-module/system.vibrator"))).then(function(vibrator) {
                                vibrator.vibrate({
                                    mode: 'long'
                                });
                            });
                        },
                        goSettings () {
                            this.$page.routeTo('pages/settings');
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
                                    value: "AI 额度"
                                }
                            }, []),
                            aiot.__ce__("text", {
                                __vm__: _vm_,
                                __opts__: {
                                    classList: [
                                        "refresh"
                                    ],
                                    events: {
                                        click: function(evt) {
                                            return _vm_.refresh(evt);
                                        }
                                    },
                                    value: "刷新"
                                }
                            }, [])
                        ]),
                        aiot.__ci__({
                            __vm__: _vm_,
                            __opts__: {
                                shown: function() {
                                    return _vm_.loaded;
                                }
                            }
                        }, function() {
                            return [
                                aiot.__ce__("div", {
                                    __vm__: _vm_,
                                    __opts__: {
                                        classList: [
                                            "card"
                                        ]
                                    }
                                }, [
                                    aiot.__cf__({
                                        __vm__: _vm_,
                                        __opts__: {
                                            exp: function() {
                                                return _vm_.limits;
                                            },
                                            key: "$idx",
                                            value: "$item"
                                        }
                                    }, function($idx, $item) {
                                        return [
                                            aiot.__ce__("div", {
                                                __vm__: _vm_,
                                                __opts__: {
                                                    classList: [
                                                        "row"
                                                    ]
                                                }
                                            }, [
                                                aiot.__ce__("text", {
                                                    __vm__: _vm_,
                                                    __opts__: {
                                                        classList: [
                                                            "label"
                                                        ],
                                                        value: function() {
                                                            return $item.label;
                                                        }
                                                    }
                                                }, []),
                                                aiot.__ce__("div", {
                                                    __vm__: _vm_,
                                                    __opts__: {
                                                        classList: [
                                                            "bar-track"
                                                        ]
                                                    }
                                                }, [
                                                    aiot.__ce__("div", {
                                                        __vm__: _vm_,
                                                        __opts__: {
                                                            classList: [
                                                                "bar-fill"
                                                            ],
                                                            style: function() {
                                                                return __webpack_require__.g.$translateStyle$("width: " + $item.width + "px");
                                                            }
                                                        }
                                                    }, [])
                                                ]),
                                                aiot.__ce__("text", {
                                                    __vm__: _vm_,
                                                    __opts__: {
                                                        classList: [
                                                            "percent"
                                                        ],
                                                        value: function() {
                                                            return $item.percent + "%";
                                                        }
                                                    }
                                                }, []),
                                                aiot.__ce__("text", {
                                                    __vm__: _vm_,
                                                    __opts__: {
                                                        classList: [
                                                            "reset"
                                                        ],
                                                        value: function() {
                                                            return $item.reset;
                                                        }
                                                    }
                                                }, [])
                                            ])
                                        ];
                                    })
                                ])
                            ];
                        }),
                        aiot.__ci__({
                            __vm__: _vm_,
                            __opts__: {
                                shown: function() {
                                    return _vm_.error;
                                }
                            }
                        }, function() {
                            return [
                                aiot.__ce__("div", {
                                    __vm__: _vm_,
                                    __opts__: {
                                        classList: [
                                            "card"
                                        ]
                                    }
                                }, [
                                    aiot.__ce__("text", {
                                        __vm__: _vm_,
                                        __opts__: {
                                            classList: [
                                                "error-text"
                                            ],
                                            value: function() {
                                                return _vm_.error;
                                            }
                                        }
                                    }, [])
                                ])
                            ];
                        }),
                        aiot.__ci__({
                            __vm__: _vm_,
                            __opts__: {
                                shown: function() {
                                    return !_vm_.loaded && !_vm_.error;
                                }
                            }
                        }, function() {
                            return [
                                aiot.__ce__("div", {
                                    __vm__: _vm_,
                                    __opts__: {
                                        classList: [
                                            "card"
                                        ]
                                    }
                                }, [
                                    aiot.__ce__("text", {
                                        __vm__: _vm_,
                                        __opts__: {
                                            classList: [
                                                "loading"
                                            ],
                                            value: "加载中…"
                                        }
                                    }, [])
                                ])
                            ];
                        }),
                        aiot.__ce__("div", {
                            __vm__: _vm_,
                            __opts__: {
                                classList: [
                                    "footer"
                                ]
                            }
                        }, [
                            aiot.__ce__("text", {
                                __vm__: _vm_,
                                __opts__: {
                                    classList: [
                                        "link"
                                    ],
                                    events: {
                                        click: function(evt) {
                                            return _vm_.goSettings(evt);
                                        }
                                    },
                                    value: "设置 API Key"
                                }
                            }, [])
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
