/**
 * webgl指令录制
 * 本模块初始化方法（initSpector）最好在第一次获取gl上下文时就调用， 否则无法保证初始化之前创建的纹理被捕获数据
 */


! function (e, t) {
    "object" == typeof exports && "object" == typeof module ? module.exports = t() : "function" == typeof define && define.amd ? define("SPECTOR", [], t) : "object" == typeof exports ? exports.SPECTOR = t() : e.SPECTOR = t()
}(self, (() => (() => {
    "use strict";
    var e = {
            d: (t, n) => {
                for (var r in n) e.o(n, r) && !e.o(t, r) && Object.defineProperty(t, r, {
                    enumerable: !0,
                    get: n[r]
                })
            },
            o: (e, t) => Object.prototype.hasOwnProperty.call(e, t),
            r: e => {
                "undefined" != typeof Symbol && Symbol.toStringTag && Object.defineProperty(e, Symbol.toStringTag, {
                    value: "Module"
                }), Object.defineProperty(e, "__esModule", {
                    value: !0
                })
            }
        },
        t = {};
    e.r(t), e.d(t, {
        Spector: () => Jt
    });
    var n, r = function () {
        function e() {}
        return e.isBuildableProgram = function (e) {
            return !!e && !!e[this.rebuildProgramFunctionName]
        }, e.rebuildProgram = function (e, t, n, r, a) {
            this.isBuildableProgram(e) && e[this.rebuildProgramFunctionName](t, n, r, a)
        }, e.rebuildProgramFunctionName = "__SPECTOR_rebuildProgram", e
    }();
    ! function (e) {
        e[e.noLog = 0] = "noLog", e[e.error = 1] = "error", e[e.warning = 2] = "warning", e[e.info = 3] = "info"
    }(n || (n = {}));
    var a, o = function () {
            function e() {}
            return e.error = function (e) {
                for (var t = [], n = 1; n < arguments.length; n++) t[n - 1] = arguments[n];
                this.level > 0 && console.error(e, t)
            }, e.warn = function (e) {
                for (var t = [], n = 1; n < arguments.length; n++) t[n - 1] = arguments[n];
                this.level > 1 && console.warn(e, t)
            }, e.info = function (e) {
                for (var t = [], n = 1; n < arguments.length; n++) t[n - 1] = arguments[n];
                this.level > 2 && console.log(e, t)
            }, e.level = n.warning, e
        }(),
        i = function () {
            function e() {
                this.callbacks = [], this.counter = -1
            }
            return e.prototype.add = function (e, t) {
                return this.counter++, t && (e = e.bind(t)), this.callbacks[this.counter] = e, this.counter
            }, e.prototype.remove = function (e) {
                delete this.callbacks[e]
            }, e.prototype.clear = function () {
                this.callbacks = {}
            }, e.prototype.trigger = function (e) {
                for (var t in this.callbacks) this.callbacks.hasOwnProperty(t) && this.callbacks[t](e)
            }, e
        }(),
        s = function () {
            function e() {
                if (window.performance && window.performance.now && performance.timing) this.nowFunction = this.dateBasedPerformanceNow.bind(this);
                else {
                    var e = new Date;
                    this.nowFunction = e.getTime.bind(e)
                }
            }
            return e.prototype.dateBasedPerformanceNow = function () {
                return performance.timing.navigationStart + performance.now()
            }, Object.defineProperty(e, "now", {
                get: function () {
                    return e.instance.nowFunction()
                },
                enumerable: !1,
                configurable: !0
            }), e.instance = new e, e
        }(),
        u = function () {
            function e(e) {
                this.options = e
            }
            return e.prototype.appendAnalysis = function (e) {
                e.analyses = e.analyses || [];
                var t = this.getAnalysis(e);
                e.analyses.push(t)
            }, e.prototype.getAnalysis = function (e) {
                var t = {
                    analyserName: this.analyserName
                };
                return this.appendToAnalysis(e, t), t
            }, e
        }(),
        c = (a = function (e, t) {
            return a = Object.setPrototypeOf || {
                __proto__: []
            }
            instanceof Array && function (e, t) {
                e.__proto__ = t
            } || function (e, t) {
                for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
            }, a(e, t)
        }, function (e, t) {
            if ("function" != typeof t && null !== t) throw new TypeError("Class extends value " + String(t) + " is not a constructor or null");

            function n() {
                this.constructor = e
            }
            a(e, t), e.prototype = null === t ? Object.create(t) : (n.prototype = t.prototype, new n)
        }),
        E = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return c(t, e), Object.defineProperty(t.prototype, "analyserName", {
                get: function () {
                    return t.analyserName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.appendToAnalysis = function (e, t) {
                if (e.commands) {
                    for (var n = {}, r = 0, a = e.commands; r < a.length; r++) {
                        var o = a[r];
                        n[o.name] = n[o.name] || 0, n[o.name]++
                    }
                    var i = Object.keys(n).map((function (e) {
                        return [e, n[e]]
                    }));
                    i.sort((function (e, t) {
                        var n = t[1] - e[1];
                        return 0 === n ? e[0].localeCompare(t[0]) : n
                    }));
                    for (var s = 0, u = i; s < u.length; s++) {
                        var c = u[s];
                        t[c[0]] = c[1]
                    }
                }
            }, t.analyserName = "Commands", t
        }(u),
        _ = ["drawArrays", "drawElements", "drawArraysInstanced", "drawArraysInstancedANGLE", "drawElementsInstanced", "drawElementsInstancedANGLE", "drawRangeElements", "multiDrawArraysInstancedBaseInstanceWEBGL", "multiDrawElementsInstancedBaseVertexBaseInstanceWEBGL"],
        p = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        l = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return p(t, e), Object.defineProperty(t.prototype, "analyserName", {
                get: function () {
                    return t.analyserName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.appendToAnalysis = function (e, t) {
                if (e.commands) {
                    t.total = e.commands.length, t.draw = 0, t.clear = 0;
                    for (var n = 0, r = e.commands; n < r.length; n++) {
                        var a = r[n];
                        "clear" === a.name ? t.clear++ : _.indexOf(a.name) > -1 && t.draw++
                    }
                }
            }, t.analyserName = "CommandsSummary", t
        }(u),
        m = function () {
            function e() {}
            return e.isWebGlConstant = function (e) {
                return null !== R[e] && void 0 !== R[e]
            }, e.stringifyWebGlConstant = function (e, t) {
                if (null == e) return "";
                if (0 === e) return this.zeroMeaningByCommand[t] || "0";
                if (1 === e) return this.oneMeaningByCommand[t] || "1";
                var n = R[e];
                return n ? n.name : e + ""
            }, e.DEPTH_BUFFER_BIT = {
                name: "DEPTH_BUFFER_BIT",
                value: 256,
                description: "Passed to clear to clear the current depth buffer."
            }, e.STENCIL_BUFFER_BIT = {
                name: "STENCIL_BUFFER_BIT",
                value: 1024,
                description: "Passed to clear to clear the current stencil buffer."
            }, e.COLOR_BUFFER_BIT = {
                name: "COLOR_BUFFER_BIT",
                value: 16384,
                description: "Passed to clear to clear the current color buffer."
            }, e.POINTS = {
                name: "POINTS",
                value: 0,
                description: "Passed to drawElements or drawArrays to draw single points."
            }, e.LINES = {
                name: "LINES",
                value: 1,
                description: "Passed to drawElements or drawArrays to draw lines. Each vertex connects to the one after it."
            }, e.LINE_LOOP = {
                name: "LINE_LOOP",
                value: 2,
                description: "Passed to drawElements or drawArrays to draw lines. Each set of two vertices is treated as a separate line segment."
            }, e.LINE_STRIP = {
                name: "LINE_STRIP",
                value: 3,
                description: "Passed to drawElements or drawArrays to draw a connected group of line segments from the first vertex to the last."
            }, e.TRIANGLES = {
                name: "TRIANGLES",
                value: 4,
                description: "Passed to drawElements or drawArrays to draw triangles. Each set of three vertices creates a separate triangle."
            }, e.TRIANGLE_STRIP = {
                name: "TRIANGLE_STRIP",
                value: 5,
                description: "Passed to drawElements or drawArrays to draw a connected group of triangles."
            }, e.TRIANGLE_FAN = {
                name: "TRIANGLE_FAN",
                value: 6,
                description: "Passed to drawElements or drawArrays to draw a connected group of triangles. Each vertex connects to the previous and the first vertex in the fan."
            }, e.ZERO = {
                name: "ZERO",
                value: 0,
                description: "Passed to blendFunc or blendFuncSeparate to turn off a component."
            }, e.ONE = {
                name: "ONE",
                value: 1,
                description: "Passed to blendFunc or blendFuncSeparate to turn on a component."
            }, e.SRC_COLOR = {
                name: "SRC_COLOR",
                value: 768,
                description: "Passed to blendFunc or blendFuncSeparate to multiply a component by the source elements color."
            }, e.ONE_MINUS_SRC_COLOR = {
                name: "ONE_MINUS_SRC_COLOR",
                value: 769,
                description: "Passed to blendFunc or blendFuncSeparate to multiply a component by one minus the source elements color."
            }, e.SRC_ALPHA = {
                name: "SRC_ALPHA",
                value: 770,
                description: "Passed to blendFunc or blendFuncSeparate to multiply a component by the source's alpha."
            }, e.ONE_MINUS_SRC_ALPHA = {
                name: "ONE_MINUS_SRC_ALPHA",
                value: 771,
                description: "Passed to blendFunc or blendFuncSeparate to multiply a component by one minus the source's alpha."
            }, e.DST_ALPHA = {
                name: "DST_ALPHA",
                value: 772,
                description: "Passed to blendFunc or blendFuncSeparate to multiply a component by the destination's alpha."
            }, e.ONE_MINUS_DST_ALPHA = {
                name: "ONE_MINUS_DST_ALPHA",
                value: 773,
                description: "Passed to blendFunc or blendFuncSeparate to multiply a component by one minus the destination's alpha."
            }, e.DST_COLOR = {
                name: "DST_COLOR",
                value: 774,
                description: "Passed to blendFunc or blendFuncSeparate to multiply a component by the destination's color."
            }, e.ONE_MINUS_DST_COLOR = {
                name: "ONE_MINUS_DST_COLOR",
                value: 775,
                description: "Passed to blendFunc or blendFuncSeparate to multiply a component by one minus the destination's color."
            }, e.SRC_ALPHA_SATURATE = {
                name: "SRC_ALPHA_SATURATE",
                value: 776,
                description: "Passed to blendFunc or blendFuncSeparate to multiply a component by the minimum of source's alpha or one minus the destination's alpha."
            }, e.CONSTANT_COLOR = {
                name: "CONSTANT_COLOR",
                value: 32769,
                description: "Passed to blendFunc or blendFuncSeparate to specify a constant color blend function."
            }, e.ONE_MINUS_CONSTANT_COLOR = {
                name: "ONE_MINUS_CONSTANT_COLOR",
                value: 32770,
                description: "Passed to blendFunc or blendFuncSeparate to specify one minus a constant color blend function."
            }, e.CONSTANT_ALPHA = {
                name: "CONSTANT_ALPHA",
                value: 32771,
                description: "Passed to blendFunc or blendFuncSeparate to specify a constant alpha blend function."
            }, e.ONE_MINUS_CONSTANT_ALPHA = {
                name: "ONE_MINUS_CONSTANT_ALPHA",
                value: 32772,
                description: "Passed to blendFunc or blendFuncSeparate to specify one minus a constant alpha blend function."
            }, e.FUNC_ADD = {
                name: "FUNC_ADD",
                value: 32774,
                description: "Passed to blendEquation or blendEquationSeparate to set an addition blend function."
            }, e.FUNC_SUBSTRACT = {
                name: "FUNC_SUBSTRACT",
                value: 32778,
                description: "Passed to blendEquation or blendEquationSeparate to specify a subtraction blend function (source - destination)."
            }, e.FUNC_REVERSE_SUBTRACT = {
                name: "FUNC_REVERSE_SUBTRACT",
                value: 32779,
                description: "Passed to blendEquation or blendEquationSeparate to specify a reverse subtraction blend function (destination - source)."
            }, e.BLEND_EQUATION = {
                name: "BLEND_EQUATION",
                value: 32777,
                description: "Passed to getParameter to get the current RGB blend function."
            }, e.BLEND_EQUATION_RGB = {
                name: "BLEND_EQUATION_RGB",
                value: 32777,
                description: "Passed to getParameter to get the current RGB blend function. Same as BLEND_EQUATION"
            }, e.BLEND_EQUATION_ALPHA = {
                name: "BLEND_EQUATION_ALPHA",
                value: 34877,
                description: "Passed to getParameter to get the current alpha blend function. Same as BLEND_EQUATION"
            }, e.BLEND_DST_RGB = {
                name: "BLEND_DST_RGB",
                value: 32968,
                description: "Passed to getParameter to get the current destination RGB blend function."
            }, e.BLEND_SRC_RGB = {
                name: "BLEND_SRC_RGB",
                value: 32969,
                description: "Passed to getParameter to get the current destination RGB blend function."
            }, e.BLEND_DST_ALPHA = {
                name: "BLEND_DST_ALPHA",
                value: 32970,
                description: "Passed to getParameter to get the current destination alpha blend function."
            }, e.BLEND_SRC_ALPHA = {
                name: "BLEND_SRC_ALPHA",
                value: 32971,
                description: "Passed to getParameter to get the current source alpha blend function."
            }, e.BLEND_COLOR = {
                name: "BLEND_COLOR",
                value: 32773,
                description: "Passed to getParameter to return a the current blend color."
            }, e.ARRAY_BUFFER_BINDING = {
                name: "ARRAY_BUFFER_BINDING",
                value: 34964,
                description: "Passed to getParameter to get the array buffer binding."
            }, e.ELEMENT_ARRAY_BUFFER_BINDING = {
                name: "ELEMENT_ARRAY_BUFFER_BINDING",
                value: 34965,
                description: "Passed to getParameter to get the current element array buffer."
            }, e.LINE_WIDTH = {
                name: "LINE_WIDTH",
                value: 2849,
                description: "Passed to getParameter to get the current lineWidth (set by the lineWidth method)."
            }, e.ALIASED_POINT_SIZE_RANGE = {
                name: "ALIASED_POINT_SIZE_RANGE",
                value: 33901,
                description: "Passed to getParameter to get the current size of a point drawn with gl.POINTS"
            }, e.ALIASED_LINE_WIDTH_RANGE = {
                name: "ALIASED_LINE_WIDTH_RANGE",
                value: 33902,
                description: "Passed to getParameter to get the range of available widths for a line. Returns a length-2 array with the lo value at 0, and hight at 1."
            }, e.CULL_FACE_MODE = {
                name: "CULL_FACE_MODE",
                value: 2885,
                description: "Passed to getParameter to get the current value of cullFace. Should return FRONT, BACK, or FRONT_AND_BACK"
            }, e.FRONT_FACE = {
                name: "FRONT_FACE",
                value: 2886,
                description: "Passed to getParameter to determine the current value of frontFace. Should return CW or CCW."
            }, e.DEPTH_RANGE = {
                name: "DEPTH_RANGE",
                value: 2928,
                description: "Passed to getParameter to return a length-2 array of floats giving the current depth range."
            }, e.DEPTH_WRITEMASK = {
                name: "DEPTH_WRITEMASK",
                value: 2930,
                description: "Passed to getParameter to determine if the depth write mask is enabled."
            }, e.DEPTH_CLEAR_VALUE = {
                name: "DEPTH_CLEAR_VALUE",
                value: 2931,
                description: "Passed to getParameter to determine the current depth clear value."
            }, e.DEPTH_FUNC = {
                name: "DEPTH_FUNC",
                value: 2932,
                description: "Passed to getParameter to get the current depth function. Returns NEVER, ALWAYS, LESS, EQUAL, LEQUAL, GREATER, GEQUAL, or NOTEQUAL."
            }, e.STENCIL_CLEAR_VALUE = {
                name: "STENCIL_CLEAR_VALUE",
                value: 2961,
                description: "Passed to getParameter to get the value the stencil will be cleared to."
            }, e.STENCIL_FUNC = {
                name: "STENCIL_FUNC",
                value: 2962,
                description: "Passed to getParameter to get the current stencil function. Returns NEVER, ALWAYS, LESS, EQUAL, LEQUAL, GREATER, GEQUAL, or NOTEQUAL."
            }, e.STENCIL_FAIL = {
                name: "STENCIL_FAIL",
                value: 2964,
                description: "Passed to getParameter to get the current stencil fail function. Should return KEEP, REPLACE, INCR, DECR, INVERT, INCR_WRAP, or DECR_WRAP."
            }, e.STENCIL_PASS_DEPTH_FAIL = {
                name: "STENCIL_PASS_DEPTH_FAIL",
                value: 2965,
                description: "Passed to getParameter to get the current stencil fail function should the depth buffer test fail. Should return KEEP, REPLACE, INCR, DECR, INVERT, INCR_WRAP, or DECR_WRAP."
            }, e.STENCIL_PASS_DEPTH_PASS = {
                name: "STENCIL_PASS_DEPTH_PASS",
                value: 2966,
                description: "Passed to getParameter to get the current stencil fail function should the depth buffer test pass. Should return KEEP, REPLACE, INCR, DECR, INVERT, INCR_WRAP, or DECR_WRAP."
            }, e.STENCIL_REF = {
                name: "STENCIL_REF",
                value: 2967,
                description: "Passed to getParameter to get the reference value used for stencil tests."
            }, e.STENCIL_VALUE_MASK = {
                name: "STENCIL_VALUE_MASK",
                value: 2963,
                description: " "
            }, e.STENCIL_WRITEMASK = {
                name: "STENCIL_WRITEMASK",
                value: 2968,
                description: " "
            }, e.STENCIL_BACK_FUNC = {
                name: "STENCIL_BACK_FUNC",
                value: 34816,
                description: " "
            }, e.STENCIL_BACK_FAIL = {
                name: "STENCIL_BACK_FAIL",
                value: 34817,
                description: " "
            }, e.STENCIL_BACK_PASS_DEPTH_FAIL = {
                name: "STENCIL_BACK_PASS_DEPTH_FAIL",
                value: 34818,
                description: " "
            }, e.STENCIL_BACK_PASS_DEPTH_PASS = {
                name: "STENCIL_BACK_PASS_DEPTH_PASS",
                value: 34819,
                description: " "
            }, e.STENCIL_BACK_REF = {
                name: "STENCIL_BACK_REF",
                value: 36003,
                description: " "
            }, e.STENCIL_BACK_VALUE_MASK = {
                name: "STENCIL_BACK_VALUE_MASK",
                value: 36004,
                description: " "
            }, e.STENCIL_BACK_WRITEMASK = {
                name: "STENCIL_BACK_WRITEMASK",
                value: 36005,
                description: " "
            }, e.VIEWPORT = {
                name: "VIEWPORT",
                value: 2978,
                description: "Returns an Int32Array with four elements for the current viewport dimensions."
            }, e.SCISSOR_BOX = {
                name: "SCISSOR_BOX",
                value: 3088,
                description: "Returns an Int32Array with four elements for the current scissor box dimensions."
            }, e.COLOR_CLEAR_VALUE = {
                name: "COLOR_CLEAR_VALUE",
                value: 3106,
                description: " "
            }, e.COLOR_WRITEMASK = {
                name: "COLOR_WRITEMASK",
                value: 3107,
                description: " "
            }, e.UNPACK_ALIGNMENT = {
                name: "UNPACK_ALIGNMENT",
                value: 3317,
                description: " "
            }, e.PACK_ALIGNMENT = {
                name: "PACK_ALIGNMENT",
                value: 3333,
                description: " "
            }, e.MAX_TEXTURE_SIZE = {
                name: "MAX_TEXTURE_SIZE",
                value: 3379,
                description: " "
            }, e.MAX_VIEWPORT_DIMS = {
                name: "MAX_VIEWPORT_DIMS",
                value: 3386,
                description: " "
            }, e.SUBPIXEL_BITS = {
                name: "SUBPIXEL_BITS",
                value: 3408,
                description: " "
            }, e.RED_BITS = {
                name: "RED_BITS",
                value: 3410,
                description: " "
            }, e.GREEN_BITS = {
                name: "GREEN_BITS",
                value: 3411,
                description: " "
            }, e.BLUE_BITS = {
                name: "BLUE_BITS",
                value: 3412,
                description: " "
            }, e.ALPHA_BITS = {
                name: "ALPHA_BITS",
                value: 3413,
                description: " "
            }, e.DEPTH_BITS = {
                name: "DEPTH_BITS",
                value: 3414,
                description: " "
            }, e.STENCIL_BITS = {
                name: "STENCIL_BITS",
                value: 3415,
                description: " "
            }, e.POLYGON_OFFSET_UNITS = {
                name: "POLYGON_OFFSET_UNITS",
                value: 10752,
                description: " "
            }, e.POLYGON_OFFSET_FACTOR = {
                name: "POLYGON_OFFSET_FACTOR",
                value: 32824,
                description: " "
            }, e.TEXTURE_BINDING_2D = {
                name: "TEXTURE_BINDING_2D",
                value: 32873,
                description: " "
            }, e.SAMPLE_BUFFERS = {
                name: "SAMPLE_BUFFERS",
                value: 32936,
                description: " "
            }, e.SAMPLES = {
                name: "SAMPLES",
                value: 32937,
                description: " "
            }, e.SAMPLE_COVERAGE_VALUE = {
                name: "SAMPLE_COVERAGE_VALUE",
                value: 32938,
                description: " "
            }, e.SAMPLE_COVERAGE_INVERT = {
                name: "SAMPLE_COVERAGE_INVERT",
                value: 32939,
                description: " "
            }, e.COMPRESSED_TEXTURE_FORMATS = {
                name: "COMPRESSED_TEXTURE_FORMATS",
                value: 34467,
                description: " "
            }, e.VENDOR = {
                name: "VENDOR",
                value: 7936,
                description: " "
            }, e.RENDERER = {
                name: "RENDERER",
                value: 7937,
                description: " "
            }, e.VERSION = {
                name: "VERSION",
                value: 7938,
                description: " "
            }, e.IMPLEMENTATION_COLOR_READ_TYPE = {
                name: "IMPLEMENTATION_COLOR_READ_TYPE",
                value: 35738,
                description: " "
            }, e.IMPLEMENTATION_COLOR_READ_FORMAT = {
                name: "IMPLEMENTATION_COLOR_READ_FORMAT",
                value: 35739,
                description: " "
            }, e.BROWSER_DEFAULT_WEBGL = {
                name: "BROWSER_DEFAULT_WEBGL",
                value: 37444,
                description: " "
            }, e.STATIC_DRAW = {
                name: "STATIC_DRAW",
                value: 35044,
                description: "Passed to bufferData as a hint about whether the contents of the buffer are likely to be used often and not change often."
            }, e.STREAM_DRAW = {
                name: "STREAM_DRAW",
                value: 35040,
                description: "Passed to bufferData as a hint about whether the contents of the buffer are likely to not be used often."
            }, e.DYNAMIC_DRAW = {
                name: "DYNAMIC_DRAW",
                value: 35048,
                description: "Passed to bufferData as a hint about whether the contents of the buffer are likely to be used often and change often."
            }, e.ARRAY_BUFFER = {
                name: "ARRAY_BUFFER",
                value: 34962,
                description: "Passed to bindBuffer or bufferData to specify the type of buffer being used."
            }, e.ELEMENT_ARRAY_BUFFER = {
                name: "ELEMENT_ARRAY_BUFFER",
                value: 34963,
                description: "Passed to bindBuffer or bufferData to specify the type of buffer being used."
            }, e.BUFFER_SIZE = {
                name: "BUFFER_SIZE",
                value: 34660,
                description: "Passed to getBufferParameter to get a buffer's size."
            }, e.BUFFER_USAGE = {
                name: "BUFFER_USAGE",
                value: 34661,
                description: "Passed to getBufferParameter to get the hint for the buffer passed in when it was created."
            }, e.CURRENT_VERTEX_ATTRIB = {
                name: "CURRENT_VERTEX_ATTRIB",
                value: 34342,
                description: "Passed to getVertexAttrib to read back the current vertex attribute."
            }, e.VERTEX_ATTRIB_ARRAY_ENABLED = {
                name: "VERTEX_ATTRIB_ARRAY_ENABLED",
                value: 34338,
                description: " "
            }, e.VERTEX_ATTRIB_ARRAY_SIZE = {
                name: "VERTEX_ATTRIB_ARRAY_SIZE",
                value: 34339,
                description: " "
            }, e.VERTEX_ATTRIB_ARRAY_STRIDE = {
                name: "VERTEX_ATTRIB_ARRAY_STRIDE",
                value: 34340,
                description: " "
            }, e.VERTEX_ATTRIB_ARRAY_TYPE = {
                name: "VERTEX_ATTRIB_ARRAY_TYPE",
                value: 34341,
                description: " "
            }, e.VERTEX_ATTRIB_ARRAY_NORMALIZED = {
                name: "VERTEX_ATTRIB_ARRAY_NORMALIZED",
                value: 34922,
                description: " "
            }, e.VERTEX_ATTRIB_ARRAY_POINTER = {
                name: "VERTEX_ATTRIB_ARRAY_POINTER",
                value: 34373,
                description: " "
            }, e.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING = {
                name: "VERTEX_ATTRIB_ARRAY_BUFFER_BINDING",
                value: 34975,
                description: " "
            }, e.CULL_FACE = {
                name: "CULL_FACE",
                value: 2884,
                description: "Passed to enable/disable to turn on/off culling. Can also be used with getParameter to find the current culling method."
            }, e.FRONT = {
                name: "FRONT",
                value: 1028,
                description: "Passed to cullFace to specify that only front faces should be drawn."
            }, e.BACK = {
                name: "BACK",
                value: 1029,
                description: "Passed to cullFace to specify that only back faces should be drawn."
            }, e.FRONT_AND_BACK = {
                name: "FRONT_AND_BACK",
                value: 1032,
                description: "Passed to cullFace to specify that front and back faces should be drawn."
            }, e.BLEND = {
                name: "BLEND",
                value: 3042,
                description: "Passed to enable/disable to turn on/off blending. Can also be used with getParameter to find the current blending method."
            }, e.DEPTH_TEST = {
                name: "DEPTH_TEST",
                value: 2929,
                description: "Passed to enable/disable to turn on/off the depth test. Can also be used with getParameter to query the depth test."
            }, e.DITHER = {
                name: "DITHER",
                value: 3024,
                description: "Passed to enable/disable to turn on/off dithering. Can also be used with getParameter to find the current dithering method."
            }, e.POLYGON_OFFSET_FILL = {
                name: "POLYGON_OFFSET_FILL",
                value: 32823,
                description: "Passed to enable/disable to turn on/off the polygon offset. Useful for rendering hidden-line images, decals, and or solids with highlighted edges. Can also be used with getParameter to query the scissor test."
            }, e.SAMPLE_ALPHA_TO_COVERAGE = {
                name: "SAMPLE_ALPHA_TO_COVERAGE",
                value: 32926,
                description: "Passed to enable/disable to turn on/off the alpha to coverage. Used in multi-sampling alpha channels."
            }, e.SAMPLE_COVERAGE = {
                name: "SAMPLE_COVERAGE",
                value: 32928,
                description: "Passed to enable/disable to turn on/off the sample coverage. Used in multi-sampling."
            }, e.SCISSOR_TEST = {
                name: "SCISSOR_TEST",
                value: 3089,
                description: "Passed to enable/disable to turn on/off the scissor test. Can also be used with getParameter to query the scissor test."
            }, e.STENCIL_TEST = {
                name: "STENCIL_TEST",
                value: 2960,
                description: "Passed to enable/disable to turn on/off the stencil test. Can also be used with getParameter to query the stencil test."
            }, e.NO_ERROR = {
                name: "NO_ERROR",
                value: 0,
                description: "Returned from getError."
            }, e.INVALID_ENUM = {
                name: "INVALID_ENUM",
                value: 1280,
                description: "Returned from getError."
            }, e.INVALID_VALUE = {
                name: "INVALID_VALUE",
                value: 1281,
                description: "Returned from getError."
            }, e.INVALID_OPERATION = {
                name: "INVALID_OPERATION",
                value: 1282,
                description: "Returned from getError."
            }, e.OUT_OF_MEMORY = {
                name: "OUT_OF_MEMORY",
                value: 1285,
                description: "Returned from getError."
            }, e.CONTEXT_LOST_WEBGL = {
                name: "CONTEXT_LOST_WEBGL",
                value: 37442,
                description: "Returned from getError."
            }, e.CW = {
                name: "CW",
                value: 2304,
                description: "Passed to frontFace to specify the front face of a polygon is drawn in the clockwise direction"
            }, e.CCW = {
                name: "CCW",
                value: 2305,
                description: "Passed to frontFace to specify the front face of a polygon is drawn in the counter clockwise direction"
            }, e.DONT_CARE = {
                name: "DONT_CARE",
                value: 4352,
                description: "There is no preference for this behavior."
            }, e.FASTEST = {
                name: "FASTEST",
                value: 4353,
                description: "The most efficient behavior should be used."
            }, e.NICEST = {
                name: "NICEST",
                value: 4354,
                description: "The most correct or the highest quality option should be used."
            }, e.GENERATE_MIPMAP_HINT = {
                name: "GENERATE_MIPMAP_HINT",
                value: 33170,
                description: "Hint for the quality of filtering when generating mipmap images with WebGLRenderingContext.generateMipmap()."
            }, e.BYTE = {
                name: "BYTE",
                value: 5120,
                description: " "
            }, e.UNSIGNED_BYTE = {
                name: "UNSIGNED_BYTE",
                value: 5121,
                description: " "
            }, e.SHORT = {
                name: "SHORT",
                value: 5122,
                description: " "
            }, e.UNSIGNED_SHORT = {
                name: "UNSIGNED_SHORT",
                value: 5123,
                description: " "
            }, e.INT = {
                name: "INT",
                value: 5124,
                description: " "
            }, e.UNSIGNED_INT = {
                name: "UNSIGNED_INT",
                value: 5125,
                description: " "
            }, e.FLOAT = {
                name: "FLOAT",
                value: 5126,
                description: " "
            }, e.DEPTH_COMPONENT = {
                name: "DEPTH_COMPONENT",
                value: 6402,
                description: " "
            }, e.ALPHA = {
                name: "ALPHA",
                value: 6406,
                description: " "
            }, e.RGB = {
                name: "RGB",
                value: 6407,
                description: " "
            }, e.RGBA = {
                name: "RGBA",
                value: 6408,
                description: " "
            }, e.LUMINANCE = {
                name: "LUMINANCE",
                value: 6409,
                description: " "
            }, e.LUMINANCE_ALPHA = {
                name: "LUMINANCE_ALPHA",
                value: 6410,
                description: " "
            }, e.UNSIGNED_SHORT_4_4_4_4 = {
                name: "UNSIGNED_SHORT_4_4_4_4",
                value: 32819,
                description: " "
            }, e.UNSIGNED_SHORT_5_5_5_1 = {
                name: "UNSIGNED_SHORT_5_5_5_1",
                value: 32820,
                description: " "
            }, e.UNSIGNED_SHORT_5_6_5 = {
                name: "UNSIGNED_SHORT_5_6_5",
                value: 33635,
                description: " "
            }, e.FRAGMENT_SHADER = {
                name: "FRAGMENT_SHADER",
                value: 35632,
                description: "Passed to createShader to define a fragment shader."
            }, e.VERTEX_SHADER = {
                name: "VERTEX_SHADER",
                value: 35633,
                description: "Passed to createShader to define a vertex shader"
            }, e.COMPILE_STATUS = {
                name: "COMPILE_STATUS",
                value: 35713,
                description: "Passed to getShaderParamter to get the status of the compilation. Returns false if the shader was not compiled. You can then query getShaderInfoLog to find the exact error"
            }, e.DELETE_STATUS = {
                name: "DELETE_STATUS",
                value: 35712,
                description: "Passed to getShaderParamter to determine if a shader was deleted via deleteShader. Returns true if it was, false otherwise."
            }, e.LINK_STATUS = {
                name: "LINK_STATUS",
                value: 35714,
                description: "Passed to getProgramParameter after calling linkProgram to determine if a program was linked correctly. Returns false if there were errors. Use getProgramInfoLog to find the exact error."
            }, e.VALIDATE_STATUS = {
                name: "VALIDATE_STATUS",
                value: 35715,
                description: "Passed to getProgramParameter after calling validateProgram to determine if it is valid. Returns false if errors were found."
            }, e.ATTACHED_SHADERS = {
                name: "ATTACHED_SHADERS",
                value: 35717,
                description: "Passed to getProgramParameter after calling attachShader to determine if the shader was attached correctly. Returns false if errors occurred."
            }, e.ACTIVE_ATTRIBUTES = {
                name: "ACTIVE_ATTRIBUTES",
                value: 35721,
                description: "Passed to getProgramParameter to get the number of attributes active in a program."
            }, e.ACTIVE_UNIFORMS = {
                name: "ACTIVE_UNIFORMS",
                value: 35718,
                description: "Passed to getProgramParamter to get the number of uniforms active in a program."
            }, e.MAX_VERTEX_ATTRIBS = {
                name: "MAX_VERTEX_ATTRIBS",
                value: 34921,
                description: " "
            }, e.MAX_VERTEX_UNIFORM_VECTORS = {
                name: "MAX_VERTEX_UNIFORM_VECTORS",
                value: 36347,
                description: " "
            }, e.MAX_VARYING_VECTORS = {
                name: "MAX_VARYING_VECTORS",
                value: 36348,
                description: " "
            }, e.MAX_COMBINED_TEXTURE_IMAGE_UNITS = {
                name: "MAX_COMBINED_TEXTURE_IMAGE_UNITS",
                value: 35661,
                description: " "
            }, e.MAX_VERTEX_TEXTURE_IMAGE_UNITS = {
                name: "MAX_VERTEX_TEXTURE_IMAGE_UNITS",
                value: 35660,
                description: " "
            }, e.MAX_TEXTURE_IMAGE_UNITS = {
                name: "MAX_TEXTURE_IMAGE_UNITS",
                value: 34930,
                description: "Implementation dependent number of maximum texture units. At least 8."
            }, e.MAX_FRAGMENT_UNIFORM_VECTORS = {
                name: "MAX_FRAGMENT_UNIFORM_VECTORS",
                value: 36349,
                description: " "
            }, e.SHADER_TYPE = {
                name: "SHADER_TYPE",
                value: 35663,
                description: " "
            }, e.SHADING_LANGUAGE_VERSION = {
                name: "SHADING_LANGUAGE_VERSION",
                value: 35724,
                description: " "
            }, e.CURRENT_PROGRAM = {
                name: "CURRENT_PROGRAM",
                value: 35725,
                description: " "
            }, e.NEVER = {
                name: "NEVER",
                value: 512,
                description: "Passed to depthFunction or stencilFunction to specify depth or stencil tests will never pass. i.e. Nothing will be drawn."
            }, e.ALWAYS = {
                name: "ALWAYS",
                value: 519,
                description: "Passed to depthFunction or stencilFunction to specify depth or stencil tests will always pass. i.e. Pixels will be drawn in the order they are drawn."
            }, e.LESS = {
                name: "LESS",
                value: 513,
                description: "Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is less than the stored value."
            }, e.EQUAL = {
                name: "EQUAL",
                value: 514,
                description: "Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is equals to the stored value."
            }, e.LEQUAL = {
                name: "LEQUAL",
                value: 515,
                description: "Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is less than or equal to the stored value."
            }, e.GREATER = {
                name: "GREATER",
                value: 516,
                description: "Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is greater than the stored value."
            }, e.GEQUAL = {
                name: "GEQUAL",
                value: 518,
                description: "Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is greater than or equal to the stored value."
            }, e.NOTEQUAL = {
                name: "NOTEQUAL",
                value: 517,
                description: "Passed to depthFunction or stencilFunction to specify depth or stencil tests will pass if the new depth value is not equal to the stored value."
            }, e.KEEP = {
                name: "KEEP",
                value: 7680,
                description: " "
            }, e.REPLACE = {
                name: "REPLACE",
                value: 7681,
                description: " "
            }, e.INCR = {
                name: "INCR",
                value: 7682,
                description: " "
            }, e.DECR = {
                name: "DECR",
                value: 7683,
                description: " "
            }, e.INVERT = {
                name: "INVERT",
                value: 5386,
                description: " "
            }, e.INCR_WRAP = {
                name: "INCR_WRAP",
                value: 34055,
                description: " "
            }, e.DECR_WRAP = {
                name: "DECR_WRAP",
                value: 34056,
                description: " "
            }, e.NEAREST = {
                name: "NEAREST",
                value: 9728,
                description: " "
            }, e.LINEAR = {
                name: "LINEAR",
                value: 9729,
                description: " "
            }, e.NEAREST_MIPMAP_NEAREST = {
                name: "NEAREST_MIPMAP_NEAREST",
                value: 9984,
                description: " "
            }, e.LINEAR_MIPMAP_NEAREST = {
                name: "LINEAR_MIPMAP_NEAREST",
                value: 9985,
                description: " "
            }, e.NEAREST_MIPMAP_LINEAR = {
                name: "NEAREST_MIPMAP_LINEAR",
                value: 9986,
                description: " "
            }, e.LINEAR_MIPMAP_LINEAR = {
                name: "LINEAR_MIPMAP_LINEAR",
                value: 9987,
                description: " "
            }, e.TEXTURE_MAG_FILTER = {
                name: "TEXTURE_MAG_FILTER",
                value: 10240,
                description: " "
            }, e.TEXTURE_MIN_FILTER = {
                name: "TEXTURE_MIN_FILTER",
                value: 10241,
                description: " "
            }, e.TEXTURE_WRAP_S = {
                name: "TEXTURE_WRAP_S",
                value: 10242,
                description: " "
            }, e.TEXTURE_WRAP_T = {
                name: "TEXTURE_WRAP_T",
                value: 10243,
                description: " "
            }, e.TEXTURE_2D = {
                name: "TEXTURE_2D",
                value: 3553,
                description: " "
            }, e.TEXTURE = {
                name: "TEXTURE",
                value: 5890,
                description: " "
            }, e.TEXTURE_CUBE_MAP = {
                name: "TEXTURE_CUBE_MAP",
                value: 34067,
                description: " "
            }, e.TEXTURE_BINDING_CUBE_MAP = {
                name: "TEXTURE_BINDING_CUBE_MAP",
                value: 34068,
                description: " "
            }, e.TEXTURE_CUBE_MAP_POSITIVE_X = {
                name: "TEXTURE_CUBE_MAP_POSITIVE_X",
                value: 34069,
                description: " "
            }, e.TEXTURE_CUBE_MAP_NEGATIVE_X = {
                name: "TEXTURE_CUBE_MAP_NEGATIVE_X",
                value: 34070,
                description: " "
            }, e.TEXTURE_CUBE_MAP_POSITIVE_Y = {
                name: "TEXTURE_CUBE_MAP_POSITIVE_Y",
                value: 34071,
                description: " "
            }, e.TEXTURE_CUBE_MAP_NEGATIVE_Y = {
                name: "TEXTURE_CUBE_MAP_NEGATIVE_Y",
                value: 34072,
                description: " "
            }, e.TEXTURE_CUBE_MAP_POSITIVE_Z = {
                name: "TEXTURE_CUBE_MAP_POSITIVE_Z",
                value: 34073,
                description: " "
            }, e.TEXTURE_CUBE_MAP_NEGATIVE_Z = {
                name: "TEXTURE_CUBE_MAP_NEGATIVE_Z",
                value: 34074,
                description: " "
            }, e.MAX_CUBE_MAP_TEXTURE_SIZE = {
                name: "MAX_CUBE_MAP_TEXTURE_SIZE",
                value: 34076,
                description: " "
            }, e.TEXTURE0 = {
                name: "TEXTURE0",
                value: 33984,
                description: "A texture unit."
            }, e.TEXTURE1 = {
                name: "TEXTURE1",
                value: 33985,
                description: "A texture unit."
            }, e.TEXTURE2 = {
                name: "TEXTURE2",
                value: 33986,
                description: "A texture unit."
            }, e.TEXTURE3 = {
                name: "TEXTURE3",
                value: 33987,
                description: "A texture unit."
            }, e.TEXTURE4 = {
                name: "TEXTURE4",
                value: 33988,
                description: "A texture unit."
            }, e.TEXTURE5 = {
                name: "TEXTURE5",
                value: 33989,
                description: "A texture unit."
            }, e.TEXTURE6 = {
                name: "TEXTURE6",
                value: 33990,
                description: "A texture unit."
            }, e.TEXTURE7 = {
                name: "TEXTURE7",
                value: 33991,
                description: "A texture unit."
            }, e.TEXTURE8 = {
                name: "TEXTURE8",
                value: 33992,
                description: "A texture unit."
            }, e.TEXTURE9 = {
                name: "TEXTURE9",
                value: 33993,
                description: "A texture unit."
            }, e.TEXTURE10 = {
                name: "TEXTURE10",
                value: 33994,
                description: "A texture unit."
            }, e.TEXTURE11 = {
                name: "TEXTURE11",
                value: 33995,
                description: "A texture unit."
            }, e.TEXTURE12 = {
                name: "TEXTURE12",
                value: 33996,
                description: "A texture unit."
            }, e.TEXTURE13 = {
                name: "TEXTURE13",
                value: 33997,
                description: "A texture unit."
            }, e.TEXTURE14 = {
                name: "TEXTURE14",
                value: 33998,
                description: "A texture unit."
            }, e.TEXTURE15 = {
                name: "TEXTURE15",
                value: 33999,
                description: "A texture unit."
            }, e.TEXTURE16 = {
                name: "TEXTURE16",
                value: 34e3,
                description: "A texture unit."
            }, e.TEXTURE17 = {
                name: "TEXTURE17",
                value: 34001,
                description: "A texture unit."
            }, e.TEXTURE18 = {
                name: "TEXTURE18",
                value: 34002,
                description: "A texture unit."
            }, e.TEXTURE19 = {
                name: "TEXTURE19",
                value: 34003,
                description: "A texture unit."
            }, e.TEXTURE20 = {
                name: "TEXTURE20",
                value: 34004,
                description: "A texture unit."
            }, e.TEXTURE21 = {
                name: "TEXTURE21",
                value: 34005,
                description: "A texture unit."
            }, e.TEXTURE22 = {
                name: "TEXTURE22",
                value: 34006,
                description: "A texture unit."
            }, e.TEXTURE23 = {
                name: "TEXTURE23",
                value: 34007,
                description: "A texture unit."
            }, e.TEXTURE24 = {
                name: "TEXTURE24",
                value: 34008,
                description: "A texture unit."
            }, e.TEXTURE25 = {
                name: "TEXTURE25",
                value: 34009,
                description: "A texture unit."
            }, e.TEXTURE26 = {
                name: "TEXTURE26",
                value: 34010,
                description: "A texture unit."
            }, e.TEXTURE27 = {
                name: "TEXTURE27",
                value: 34011,
                description: "A texture unit."
            }, e.TEXTURE28 = {
                name: "TEXTURE28",
                value: 34012,
                description: "A texture unit."
            }, e.TEXTURE29 = {
                name: "TEXTURE29",
                value: 34013,
                description: "A texture unit."
            }, e.TEXTURE30 = {
                name: "TEXTURE30",
                value: 34014,
                description: "A texture unit."
            }, e.TEXTURE31 = {
                name: "TEXTURE31",
                value: 34015,
                description: "A texture unit."
            }, e.ACTIVE_TEXTURE = {
                name: "ACTIVE_TEXTURE",
                value: 34016,
                description: "The current active texture unit."
            }, e.REPEAT = {
                name: "REPEAT",
                value: 10497,
                description: " "
            }, e.CLAMP_TO_EDGE = {
                name: "CLAMP_TO_EDGE",
                value: 33071,
                description: " "
            }, e.MIRRORED_REPEAT = {
                name: "MIRRORED_REPEAT",
                value: 33648,
                description: " "
            }, e.FLOAT_VEC2 = {
                name: "FLOAT_VEC2",
                value: 35664,
                description: " "
            }, e.FLOAT_VEC3 = {
                name: "FLOAT_VEC3",
                value: 35665,
                description: " "
            }, e.FLOAT_VEC4 = {
                name: "FLOAT_VEC4",
                value: 35666,
                description: " "
            }, e.INT_VEC2 = {
                name: "INT_VEC2",
                value: 35667,
                description: " "
            }, e.INT_VEC3 = {
                name: "INT_VEC3",
                value: 35668,
                description: " "
            }, e.INT_VEC4 = {
                name: "INT_VEC4",
                value: 35669,
                description: " "
            }, e.BOOL = {
                name: "BOOL",
                value: 35670,
                description: " "
            }, e.BOOL_VEC2 = {
                name: "BOOL_VEC2",
                value: 35671,
                description: " "
            }, e.BOOL_VEC3 = {
                name: "BOOL_VEC3",
                value: 35672,
                description: " "
            }, e.BOOL_VEC4 = {
                name: "BOOL_VEC4",
                value: 35673,
                description: " "
            }, e.FLOAT_MAT2 = {
                name: "FLOAT_MAT2",
                value: 35674,
                description: " "
            }, e.FLOAT_MAT3 = {
                name: "FLOAT_MAT3",
                value: 35675,
                description: " "
            }, e.FLOAT_MAT4 = {
                name: "FLOAT_MAT4",
                value: 35676,
                description: " "
            }, e.SAMPLER_2D = {
                name: "SAMPLER_2D",
                value: 35678,
                description: " "
            }, e.SAMPLER_CUBE = {
                name: "SAMPLER_CUBE",
                value: 35680,
                description: " "
            }, e.LOW_FLOAT = {
                name: "LOW_FLOAT",
                value: 36336,
                description: " "
            }, e.MEDIUM_FLOAT = {
                name: "MEDIUM_FLOAT",
                value: 36337,
                description: " "
            }, e.HIGH_FLOAT = {
                name: "HIGH_FLOAT",
                value: 36338,
                description: " "
            }, e.LOW_INT = {
                name: "LOW_INT",
                value: 36339,
                description: " "
            }, e.MEDIUM_INT = {
                name: "MEDIUM_INT",
                value: 36340,
                description: " "
            }, e.HIGH_INT = {
                name: "HIGH_INT",
                value: 36341,
                description: " "
            }, e.FRAMEBUFFER = {
                name: "FRAMEBUFFER",
                value: 36160,
                description: " "
            }, e.RENDERBUFFER = {
                name: "RENDERBUFFER",
                value: 36161,
                description: " "
            }, e.RGBA4 = {
                name: "RGBA4",
                value: 32854,
                description: " "
            }, e.RGB5_A1 = {
                name: "RGB5_A1",
                value: 32855,
                description: " "
            }, e.RGB565 = {
                name: "RGB565",
                value: 36194,
                description: " "
            }, e.DEPTH_COMPONENT16 = {
                name: "DEPTH_COMPONENT16",
                value: 33189,
                description: " "
            }, e.STENCIL_INDEX = {
                name: "STENCIL_INDEX",
                value: 6401,
                description: " "
            }, e.STENCIL_INDEX8 = {
                name: "STENCIL_INDEX8",
                value: 36168,
                description: " "
            }, e.DEPTH_STENCIL = {
                name: "DEPTH_STENCIL",
                value: 34041,
                description: " "
            }, e.RENDERBUFFER_WIDTH = {
                name: "RENDERBUFFER_WIDTH",
                value: 36162,
                description: " "
            }, e.RENDERBUFFER_HEIGHT = {
                name: "RENDERBUFFER_HEIGHT",
                value: 36163,
                description: " "
            }, e.RENDERBUFFER_INTERNAL_FORMAT = {
                name: "RENDERBUFFER_INTERNAL_FORMAT",
                value: 36164,
                description: " "
            }, e.RENDERBUFFER_RED_SIZE = {
                name: "RENDERBUFFER_RED_SIZE",
                value: 36176,
                description: " "
            }, e.RENDERBUFFER_GREEN_SIZE = {
                name: "RENDERBUFFER_GREEN_SIZE",
                value: 36177,
                description: " "
            }, e.RENDERBUFFER_BLUE_SIZE = {
                name: "RENDERBUFFER_BLUE_SIZE",
                value: 36178,
                description: " "
            }, e.RENDERBUFFER_ALPHA_SIZE = {
                name: "RENDERBUFFER_ALPHA_SIZE",
                value: 36179,
                description: " "
            }, e.RENDERBUFFER_DEPTH_SIZE = {
                name: "RENDERBUFFER_DEPTH_SIZE",
                value: 36180,
                description: " "
            }, e.RENDERBUFFER_STENCIL_SIZE = {
                name: "RENDERBUFFER_STENCIL_SIZE",
                value: 36181,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE = {
                name: "FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE",
                value: 36048,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME = {
                name: "FRAMEBUFFER_ATTACHMENT_OBJECT_NAME",
                value: 36049,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL = {
                name: "FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL",
                value: 36050,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE = {
                name: "FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE",
                value: 36051,
                description: " "
            }, e.COLOR_ATTACHMENT0 = {
                name: "COLOR_ATTACHMENT0",
                value: 36064,
                description: " "
            }, e.DEPTH_ATTACHMENT = {
                name: "DEPTH_ATTACHMENT",
                value: 36096,
                description: " "
            }, e.STENCIL_ATTACHMENT = {
                name: "STENCIL_ATTACHMENT",
                value: 36128,
                description: " "
            }, e.DEPTH_STENCIL_ATTACHMENT = {
                name: "DEPTH_STENCIL_ATTACHMENT",
                value: 33306,
                description: " "
            }, e.NONE = {
                name: "NONE",
                value: 0,
                description: " "
            }, e.FRAMEBUFFER_COMPLETE = {
                name: "FRAMEBUFFER_COMPLETE",
                value: 36053,
                description: " "
            }, e.FRAMEBUFFER_INCOMPLETE_ATTACHMENT = {
                name: "FRAMEBUFFER_INCOMPLETE_ATTACHMENT",
                value: 36054,
                description: " "
            }, e.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT = {
                name: "FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT",
                value: 36055,
                description: " "
            }, e.FRAMEBUFFER_INCOMPLETE_DIMENSIONS = {
                name: "FRAMEBUFFER_INCOMPLETE_DIMENSIONS",
                value: 36057,
                description: " "
            }, e.FRAMEBUFFER_UNSUPPORTED = {
                name: "FRAMEBUFFER_UNSUPPORTED",
                value: 36061,
                description: " "
            }, e.FRAMEBUFFER_BINDING = {
                name: "FRAMEBUFFER_BINDING",
                value: 36006,
                description: " "
            }, e.RENDERBUFFER_BINDING = {
                name: "RENDERBUFFER_BINDING",
                value: 36007,
                description: " "
            }, e.MAX_RENDERBUFFER_SIZE = {
                name: "MAX_RENDERBUFFER_SIZE",
                value: 34024,
                description: " "
            }, e.INVALID_FRAMEBUFFER_OPERATION = {
                name: "INVALID_FRAMEBUFFER_OPERATION",
                value: 1286,
                description: " "
            }, e.UNPACK_FLIP_Y_WEBGL = {
                name: "UNPACK_FLIP_Y_WEBGL",
                value: 37440,
                description: " "
            }, e.UNPACK_PREMULTIPLY_ALPHA_WEBGL = {
                name: "UNPACK_PREMULTIPLY_ALPHA_WEBGL",
                value: 37441,
                description: " "
            }, e.UNPACK_COLORSPACE_CONVERSION_WEBGL = {
                name: "UNPACK_COLORSPACE_CONVERSION_WEBGL",
                value: 37443,
                description: " "
            }, e.READ_BUFFER = {
                name: "READ_BUFFER",
                value: 3074,
                description: " "
            }, e.UNPACK_ROW_LENGTH = {
                name: "UNPACK_ROW_LENGTH",
                value: 3314,
                description: " "
            }, e.UNPACK_SKIP_ROWS = {
                name: "UNPACK_SKIP_ROWS",
                value: 3315,
                description: " "
            }, e.UNPACK_SKIP_PIXELS = {
                name: "UNPACK_SKIP_PIXELS",
                value: 3316,
                description: " "
            }, e.PACK_ROW_LENGTH = {
                name: "PACK_ROW_LENGTH",
                value: 3330,
                description: " "
            }, e.PACK_SKIP_ROWS = {
                name: "PACK_SKIP_ROWS",
                value: 3331,
                description: " "
            }, e.PACK_SKIP_PIXELS = {
                name: "PACK_SKIP_PIXELS",
                value: 3332,
                description: " "
            }, e.TEXTURE_BINDING_3D = {
                name: "TEXTURE_BINDING_3D",
                value: 32874,
                description: " "
            }, e.UNPACK_SKIP_IMAGES = {
                name: "UNPACK_SKIP_IMAGES",
                value: 32877,
                description: " "
            }, e.UNPACK_IMAGE_HEIGHT = {
                name: "UNPACK_IMAGE_HEIGHT",
                value: 32878,
                description: " "
            }, e.MAX_3D_TEXTURE_SIZE = {
                name: "MAX_3D_TEXTURE_SIZE",
                value: 32883,
                description: " "
            }, e.MAX_ELEMENTS_VERTICES = {
                name: "MAX_ELEMENTS_VERTICES",
                value: 33e3,
                description: " "
            }, e.MAX_ELEMENTS_INDICES = {
                name: "MAX_ELEMENTS_INDICES",
                value: 33001,
                description: " "
            }, e.MAX_TEXTURE_LOD_BIAS = {
                name: "MAX_TEXTURE_LOD_BIAS",
                value: 34045,
                description: " "
            }, e.MAX_FRAGMENT_UNIFORM_COMPONENTS = {
                name: "MAX_FRAGMENT_UNIFORM_COMPONENTS",
                value: 35657,
                description: " "
            }, e.MAX_VERTEX_UNIFORM_COMPONENTS = {
                name: "MAX_VERTEX_UNIFORM_COMPONENTS",
                value: 35658,
                description: " "
            }, e.MAX_ARRAY_TEXTURE_LAYERS = {
                name: "MAX_ARRAY_TEXTURE_LAYERS",
                value: 35071,
                description: " "
            }, e.MIN_PROGRAM_TEXEL_OFFSET = {
                name: "MIN_PROGRAM_TEXEL_OFFSET",
                value: 35076,
                description: " "
            }, e.MAX_PROGRAM_TEXEL_OFFSET = {
                name: "MAX_PROGRAM_TEXEL_OFFSET",
                value: 35077,
                description: " "
            }, e.MAX_VARYING_COMPONENTS = {
                name: "MAX_VARYING_COMPONENTS",
                value: 35659,
                description: " "
            }, e.FRAGMENT_SHADER_DERIVATIVE_HINT = {
                name: "FRAGMENT_SHADER_DERIVATIVE_HINT",
                value: 35723,
                description: " "
            }, e.RASTERIZER_DISCARD = {
                name: "RASTERIZER_DISCARD",
                value: 35977,
                description: " "
            }, e.VERTEX_ARRAY_BINDING = {
                name: "VERTEX_ARRAY_BINDING",
                value: 34229,
                description: " "
            }, e.MAX_VERTEX_OUTPUT_COMPONENTS = {
                name: "MAX_VERTEX_OUTPUT_COMPONENTS",
                value: 37154,
                description: " "
            }, e.MAX_FRAGMENT_INPUT_COMPONENTS = {
                name: "MAX_FRAGMENT_INPUT_COMPONENTS",
                value: 37157,
                description: " "
            }, e.MAX_SERVER_WAIT_TIMEOUT = {
                name: "MAX_SERVER_WAIT_TIMEOUT",
                value: 37137,
                description: " "
            }, e.MAX_ELEMENT_INDEX = {
                name: "MAX_ELEMENT_INDEX",
                value: 36203,
                description: " "
            }, e.RED = {
                name: "RED",
                value: 6403,
                description: " "
            }, e.RGB8 = {
                name: "RGB8",
                value: 32849,
                description: " "
            }, e.RGBA8 = {
                name: "RGBA8",
                value: 32856,
                description: " "
            }, e.RGB10_A2 = {
                name: "RGB10_A2",
                value: 32857,
                description: " "
            }, e.TEXTURE_3D = {
                name: "TEXTURE_3D",
                value: 32879,
                description: " "
            }, e.TEXTURE_WRAP_R = {
                name: "TEXTURE_WRAP_R",
                value: 32882,
                description: " "
            }, e.TEXTURE_MIN_LOD = {
                name: "TEXTURE_MIN_LOD",
                value: 33082,
                description: " "
            }, e.TEXTURE_MAX_LOD = {
                name: "TEXTURE_MAX_LOD",
                value: 33083,
                description: " "
            }, e.TEXTURE_BASE_LEVEL = {
                name: "TEXTURE_BASE_LEVEL",
                value: 33084,
                description: " "
            }, e.TEXTURE_MAX_LEVEL = {
                name: "TEXTURE_MAX_LEVEL",
                value: 33085,
                description: " "
            }, e.TEXTURE_COMPARE_MODE = {
                name: "TEXTURE_COMPARE_MODE",
                value: 34892,
                description: " "
            }, e.TEXTURE_COMPARE_FUNC = {
                name: "TEXTURE_COMPARE_FUNC",
                value: 34893,
                description: " "
            }, e.SRGB = {
                name: "SRGB",
                value: 35904,
                description: " "
            }, e.SRGB8 = {
                name: "SRGB8",
                value: 35905,
                description: " "
            }, e.SRGB8_ALPHA8 = {
                name: "SRGB8_ALPHA8",
                value: 35907,
                description: " "
            }, e.COMPARE_REF_TO_TEXTURE = {
                name: "COMPARE_REF_TO_TEXTURE",
                value: 34894,
                description: " "
            }, e.RGBA32F = {
                name: "RGBA32F",
                value: 34836,
                description: " "
            }, e.RGB32F = {
                name: "RGB32F",
                value: 34837,
                description: " "
            }, e.RGBA16F = {
                name: "RGBA16F",
                value: 34842,
                description: " "
            }, e.RGB16F = {
                name: "RGB16F",
                value: 34843,
                description: " "
            }, e.TEXTURE_2D_ARRAY = {
                name: "TEXTURE_2D_ARRAY",
                value: 35866,
                description: " "
            }, e.TEXTURE_BINDING_2D_ARRAY = {
                name: "TEXTURE_BINDING_2D_ARRAY",
                value: 35869,
                description: " "
            }, e.R11F_G11F_B10F = {
                name: "R11F_G11F_B10F",
                value: 35898,
                description: " "
            }, e.RGB9_E5 = {
                name: "RGB9_E5",
                value: 35901,
                description: " "
            }, e.RGBA32UI = {
                name: "RGBA32UI",
                value: 36208,
                description: " "
            }, e.RGB32UI = {
                name: "RGB32UI",
                value: 36209,
                description: " "
            }, e.RGBA16UI = {
                name: "RGBA16UI",
                value: 36214,
                description: " "
            }, e.RGB16UI = {
                name: "RGB16UI",
                value: 36215,
                description: " "
            }, e.RGBA8UI = {
                name: "RGBA8UI",
                value: 36220,
                description: " "
            }, e.RGB8UI = {
                name: "RGB8UI",
                value: 36221,
                description: " "
            }, e.RGBA32I = {
                name: "RGBA32I",
                value: 36226,
                description: " "
            }, e.RGB32I = {
                name: "RGB32I",
                value: 36227,
                description: " "
            }, e.RGBA16I = {
                name: "RGBA16I",
                value: 36232,
                description: " "
            }, e.RGB16I = {
                name: "RGB16I",
                value: 36233,
                description: " "
            }, e.RGBA8I = {
                name: "RGBA8I",
                value: 36238,
                description: " "
            }, e.RGB8I = {
                name: "RGB8I",
                value: 36239,
                description: " "
            }, e.RED_INTEGER = {
                name: "RED_INTEGER",
                value: 36244,
                description: " "
            }, e.RGB_INTEGER = {
                name: "RGB_INTEGER",
                value: 36248,
                description: " "
            }, e.RGBA_INTEGER = {
                name: "RGBA_INTEGER",
                value: 36249,
                description: " "
            }, e.R8 = {
                name: "R8",
                value: 33321,
                description: " "
            }, e.RG8 = {
                name: "RG8",
                value: 33323,
                description: " "
            }, e.R16F = {
                name: "R16F",
                value: 33325,
                description: " "
            }, e.R32F = {
                name: "R32F",
                value: 33326,
                description: " "
            }, e.RG16F = {
                name: "RG16F",
                value: 33327,
                description: " "
            }, e.RG32F = {
                name: "RG32F",
                value: 33328,
                description: " "
            }, e.R8I = {
                name: "R8I",
                value: 33329,
                description: " "
            }, e.R8UI = {
                name: "R8UI",
                value: 33330,
                description: " "
            }, e.R16I = {
                name: "R16I",
                value: 33331,
                description: " "
            }, e.R16UI = {
                name: "R16UI",
                value: 33332,
                description: " "
            }, e.R32I = {
                name: "R32I",
                value: 33333,
                description: " "
            }, e.R32UI = {
                name: "R32UI",
                value: 33334,
                description: " "
            }, e.RG8I = {
                name: "RG8I",
                value: 33335,
                description: " "
            }, e.RG8UI = {
                name: "RG8UI",
                value: 33336,
                description: " "
            }, e.RG16I = {
                name: "RG16I",
                value: 33337,
                description: " "
            }, e.RG16UI = {
                name: "RG16UI",
                value: 33338,
                description: " "
            }, e.RG32I = {
                name: "RG32I",
                value: 33339,
                description: " "
            }, e.RG32UI = {
                name: "RG32UI",
                value: 33340,
                description: " "
            }, e.R8_SNORM = {
                name: "R8_SNORM",
                value: 36756,
                description: " "
            }, e.RG8_SNORM = {
                name: "RG8_SNORM",
                value: 36757,
                description: " "
            }, e.RGB8_SNORM = {
                name: "RGB8_SNORM",
                value: 36758,
                description: " "
            }, e.RGBA8_SNORM = {
                name: "RGBA8_SNORM",
                value: 36759,
                description: " "
            }, e.RGB10_A2UI = {
                name: "RGB10_A2UI",
                value: 36975,
                description: " "
            }, e.TEXTURE_IMMUTABLE_FORMAT = {
                name: "TEXTURE_IMMUTABLE_FORMAT",
                value: 37167,
                description: " "
            }, e.TEXTURE_IMMUTABLE_LEVELS = {
                name: "TEXTURE_IMMUTABLE_LEVELS",
                value: 33503,
                description: " "
            }, e.UNSIGNED_INT_2_10_10_10_REV = {
                name: "UNSIGNED_INT_2_10_10_10_REV",
                value: 33640,
                description: " "
            }, e.UNSIGNED_INT_10F_11F_11F_REV = {
                name: "UNSIGNED_INT_10F_11F_11F_REV",
                value: 35899,
                description: " "
            }, e.UNSIGNED_INT_5_9_9_9_REV = {
                name: "UNSIGNED_INT_5_9_9_9_REV",
                value: 35902,
                description: " "
            }, e.FLOAT_32_UNSIGNED_INT_24_8_REV = {
                name: "FLOAT_32_UNSIGNED_INT_24_8_REV",
                value: 36269,
                description: " "
            }, e.UNSIGNED_INT_24_8 = {
                name: "UNSIGNED_INT_24_8",
                value: 34042,
                description: " "
            }, e.HALF_FLOAT = {
                name: "HALF_FLOAT",
                value: 5131,
                description: " "
            }, e.RG = {
                name: "RG",
                value: 33319,
                description: " "
            }, e.RG_INTEGER = {
                name: "RG_INTEGER",
                value: 33320,
                description: " "
            }, e.INT_2_10_10_10_REV = {
                name: "INT_2_10_10_10_REV",
                value: 36255,
                description: " "
            }, e.CURRENT_QUERY = {
                name: "CURRENT_QUERY",
                value: 34917,
                description: " "
            }, e.QUERY_RESULT = {
                name: "QUERY_RESULT",
                value: 34918,
                description: " "
            }, e.QUERY_RESULT_AVAILABLE = {
                name: "QUERY_RESULT_AVAILABLE",
                value: 34919,
                description: " "
            }, e.ANY_SAMPLES_PASSED = {
                name: "ANY_SAMPLES_PASSED",
                value: 35887,
                description: " "
            }, e.ANY_SAMPLES_PASSED_CONSERVATIVE = {
                name: "ANY_SAMPLES_PASSED_CONSERVATIVE",
                value: 36202,
                description: " "
            }, e.MAX_DRAW_BUFFERS = {
                name: "MAX_DRAW_BUFFERS",
                value: 34852,
                description: " "
            }, e.DRAW_BUFFER0 = {
                name: "DRAW_BUFFER0",
                value: 34853,
                description: " "
            }, e.DRAW_BUFFER1 = {
                name: "DRAW_BUFFER1",
                value: 34854,
                description: " "
            }, e.DRAW_BUFFER2 = {
                name: "DRAW_BUFFER2",
                value: 34855,
                description: " "
            }, e.DRAW_BUFFER3 = {
                name: "DRAW_BUFFER3",
                value: 34856,
                description: " "
            }, e.DRAW_BUFFER4 = {
                name: "DRAW_BUFFER4",
                value: 34857,
                description: " "
            }, e.DRAW_BUFFER5 = {
                name: "DRAW_BUFFER5",
                value: 34858,
                description: " "
            }, e.DRAW_BUFFER6 = {
                name: "DRAW_BUFFER6",
                value: 34859,
                description: " "
            }, e.DRAW_BUFFER7 = {
                name: "DRAW_BUFFER7",
                value: 34860,
                description: " "
            }, e.DRAW_BUFFER8 = {
                name: "DRAW_BUFFER8",
                value: 34861,
                description: " "
            }, e.DRAW_BUFFER9 = {
                name: "DRAW_BUFFER9",
                value: 34862,
                description: " "
            }, e.DRAW_BUFFER10 = {
                name: "DRAW_BUFFER10",
                value: 34863,
                description: " "
            }, e.DRAW_BUFFER11 = {
                name: "DRAW_BUFFER11",
                value: 34864,
                description: " "
            }, e.DRAW_BUFFER12 = {
                name: "DRAW_BUFFER12",
                value: 34865,
                description: " "
            }, e.DRAW_BUFFER13 = {
                name: "DRAW_BUFFER13",
                value: 34866,
                description: " "
            }, e.DRAW_BUFFER14 = {
                name: "DRAW_BUFFER14",
                value: 34867,
                description: " "
            }, e.DRAW_BUFFER15 = {
                name: "DRAW_BUFFER15",
                value: 34868,
                description: " "
            }, e.MAX_COLOR_ATTACHMENTS = {
                name: "MAX_COLOR_ATTACHMENTS",
                value: 36063,
                description: " "
            }, e.COLOR_ATTACHMENT1 = {
                name: "COLOR_ATTACHMENT1",
                value: 36065,
                description: " "
            }, e.COLOR_ATTACHMENT2 = {
                name: "COLOR_ATTACHMENT2",
                value: 36066,
                description: " "
            }, e.COLOR_ATTACHMENT3 = {
                name: "COLOR_ATTACHMENT3",
                value: 36067,
                description: " "
            }, e.COLOR_ATTACHMENT4 = {
                name: "COLOR_ATTACHMENT4",
                value: 36068,
                description: " "
            }, e.COLOR_ATTACHMENT5 = {
                name: "COLOR_ATTACHMENT5",
                value: 36069,
                description: " "
            }, e.COLOR_ATTACHMENT6 = {
                name: "COLOR_ATTACHMENT6",
                value: 36070,
                description: " "
            }, e.COLOR_ATTACHMENT7 = {
                name: "COLOR_ATTACHMENT7",
                value: 36071,
                description: " "
            }, e.COLOR_ATTACHMENT8 = {
                name: "COLOR_ATTACHMENT8",
                value: 36072,
                description: " "
            }, e.COLOR_ATTACHMENT9 = {
                name: "COLOR_ATTACHMENT9",
                value: 36073,
                description: " "
            }, e.COLOR_ATTACHMENT10 = {
                name: "COLOR_ATTACHMENT10",
                value: 36074,
                description: " "
            }, e.COLOR_ATTACHMENT11 = {
                name: "COLOR_ATTACHMENT11",
                value: 36075,
                description: " "
            }, e.COLOR_ATTACHMENT12 = {
                name: "COLOR_ATTACHMENT12",
                value: 36076,
                description: " "
            }, e.COLOR_ATTACHMENT13 = {
                name: "COLOR_ATTACHMENT13",
                value: 36077,
                description: " "
            }, e.COLOR_ATTACHMENT14 = {
                name: "COLOR_ATTACHMENT14",
                value: 36078,
                description: " "
            }, e.COLOR_ATTACHMENT15 = {
                name: "COLOR_ATTACHMENT15",
                value: 36079,
                description: " "
            }, e.SAMPLER_3D = {
                name: "SAMPLER_3D",
                value: 35679,
                description: " "
            }, e.SAMPLER_2D_SHADOW = {
                name: "SAMPLER_2D_SHADOW",
                value: 35682,
                description: " "
            }, e.SAMPLER_2D_ARRAY = {
                name: "SAMPLER_2D_ARRAY",
                value: 36289,
                description: " "
            }, e.SAMPLER_2D_ARRAY_SHADOW = {
                name: "SAMPLER_2D_ARRAY_SHADOW",
                value: 36292,
                description: " "
            }, e.SAMPLER_CUBE_SHADOW = {
                name: "SAMPLER_CUBE_SHADOW",
                value: 36293,
                description: " "
            }, e.INT_SAMPLER_2D = {
                name: "INT_SAMPLER_2D",
                value: 36298,
                description: " "
            }, e.INT_SAMPLER_3D = {
                name: "INT_SAMPLER_3D",
                value: 36299,
                description: " "
            }, e.INT_SAMPLER_CUBE = {
                name: "INT_SAMPLER_CUBE",
                value: 36300,
                description: " "
            }, e.INT_SAMPLER_2D_ARRAY = {
                name: "INT_SAMPLER_2D_ARRAY",
                value: 36303,
                description: " "
            }, e.UNSIGNED_INT_SAMPLER_2D = {
                name: "UNSIGNED_INT_SAMPLER_2D",
                value: 36306,
                description: " "
            }, e.UNSIGNED_INT_SAMPLER_3D = {
                name: "UNSIGNED_INT_SAMPLER_3D",
                value: 36307,
                description: " "
            }, e.UNSIGNED_INT_SAMPLER_CUBE = {
                name: "UNSIGNED_INT_SAMPLER_CUBE",
                value: 36308,
                description: " "
            }, e.UNSIGNED_INT_SAMPLER_2D_ARRAY = {
                name: "UNSIGNED_INT_SAMPLER_2D_ARRAY",
                value: 36311,
                description: " "
            }, e.MAX_SAMPLES = {
                name: "MAX_SAMPLES",
                value: 36183,
                description: " "
            }, e.SAMPLER_BINDING = {
                name: "SAMPLER_BINDING",
                value: 35097,
                description: " "
            }, e.PIXEL_PACK_BUFFER = {
                name: "PIXEL_PACK_BUFFER",
                value: 35051,
                description: " "
            }, e.PIXEL_UNPACK_BUFFER = {
                name: "PIXEL_UNPACK_BUFFER",
                value: 35052,
                description: " "
            }, e.PIXEL_PACK_BUFFER_BINDING = {
                name: "PIXEL_PACK_BUFFER_BINDING",
                value: 35053,
                description: " "
            }, e.PIXEL_UNPACK_BUFFER_BINDING = {
                name: "PIXEL_UNPACK_BUFFER_BINDING",
                value: 35055,
                description: " "
            }, e.COPY_READ_BUFFER = {
                name: "COPY_READ_BUFFER",
                value: 36662,
                description: " "
            }, e.COPY_WRITE_BUFFER = {
                name: "COPY_WRITE_BUFFER",
                value: 36663,
                description: " "
            }, e.COPY_READ_BUFFER_BINDING = {
                name: "COPY_READ_BUFFER_BINDING",
                value: 36662,
                description: " "
            }, e.COPY_WRITE_BUFFER_BINDING = {
                name: "COPY_WRITE_BUFFER_BINDING",
                value: 36663,
                description: " "
            }, e.FLOAT_MAT2x3 = {
                name: "FLOAT_MAT2x3",
                value: 35685,
                description: " "
            }, e.FLOAT_MAT2x4 = {
                name: "FLOAT_MAT2x4",
                value: 35686,
                description: " "
            }, e.FLOAT_MAT3x2 = {
                name: "FLOAT_MAT3x2",
                value: 35687,
                description: " "
            }, e.FLOAT_MAT3x4 = {
                name: "FLOAT_MAT3x4",
                value: 35688,
                description: " "
            }, e.FLOAT_MAT4x2 = {
                name: "FLOAT_MAT4x2",
                value: 35689,
                description: " "
            }, e.FLOAT_MAT4x3 = {
                name: "FLOAT_MAT4x3",
                value: 35690,
                description: " "
            }, e.UNSIGNED_INT_VEC2 = {
                name: "UNSIGNED_INT_VEC2",
                value: 36294,
                description: " "
            }, e.UNSIGNED_INT_VEC3 = {
                name: "UNSIGNED_INT_VEC3",
                value: 36295,
                description: " "
            }, e.UNSIGNED_INT_VEC4 = {
                name: "UNSIGNED_INT_VEC4",
                value: 36296,
                description: " "
            }, e.UNSIGNED_NORMALIZED = {
                name: "UNSIGNED_NORMALIZED",
                value: 35863,
                description: " "
            }, e.SIGNED_NORMALIZED = {
                name: "SIGNED_NORMALIZED",
                value: 36764,
                description: " "
            }, e.VERTEX_ATTRIB_ARRAY_INTEGER = {
                name: "VERTEX_ATTRIB_ARRAY_INTEGER",
                value: 35069,
                description: " "
            }, e.VERTEX_ATTRIB_ARRAY_DIVISOR = {
                name: "VERTEX_ATTRIB_ARRAY_DIVISOR",
                value: 35070,
                description: " "
            }, e.TRANSFORM_FEEDBACK_BUFFER_MODE = {
                name: "TRANSFORM_FEEDBACK_BUFFER_MODE",
                value: 35967,
                description: " "
            }, e.MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS = {
                name: "MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS",
                value: 35968,
                description: " "
            }, e.TRANSFORM_FEEDBACK_VARYINGS = {
                name: "TRANSFORM_FEEDBACK_VARYINGS",
                value: 35971,
                description: " "
            }, e.TRANSFORM_FEEDBACK_BUFFER_START = {
                name: "TRANSFORM_FEEDBACK_BUFFER_START",
                value: 35972,
                description: " "
            }, e.TRANSFORM_FEEDBACK_BUFFER_SIZE = {
                name: "TRANSFORM_FEEDBACK_BUFFER_SIZE",
                value: 35973,
                description: " "
            }, e.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN = {
                name: "TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN",
                value: 35976,
                description: " "
            }, e.MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS = {
                name: "MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS",
                value: 35978,
                description: " "
            }, e.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS = {
                name: "MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS",
                value: 35979,
                description: " "
            }, e.INTERLEAVED_ATTRIBS = {
                name: "INTERLEAVED_ATTRIBS",
                value: 35980,
                description: " "
            }, e.SEPARATE_ATTRIBS = {
                name: "SEPARATE_ATTRIBS",
                value: 35981,
                description: " "
            }, e.TRANSFORM_FEEDBACK_BUFFER = {
                name: "TRANSFORM_FEEDBACK_BUFFER",
                value: 35982,
                description: " "
            }, e.TRANSFORM_FEEDBACK_BUFFER_BINDING = {
                name: "TRANSFORM_FEEDBACK_BUFFER_BINDING",
                value: 35983,
                description: " "
            }, e.TRANSFORM_FEEDBACK = {
                name: "TRANSFORM_FEEDBACK",
                value: 36386,
                description: " "
            }, e.TRANSFORM_FEEDBACK_PAUSED = {
                name: "TRANSFORM_FEEDBACK_PAUSED",
                value: 36387,
                description: " "
            }, e.TRANSFORM_FEEDBACK_ACTIVE = {
                name: "TRANSFORM_FEEDBACK_ACTIVE",
                value: 36388,
                description: " "
            }, e.TRANSFORM_FEEDBACK_BINDING = {
                name: "TRANSFORM_FEEDBACK_BINDING",
                value: 36389,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING = {
                name: "FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING",
                value: 33296,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE = {
                name: "FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE",
                value: 33297,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_RED_SIZE = {
                name: "FRAMEBUFFER_ATTACHMENT_RED_SIZE",
                value: 33298,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_GREEN_SIZE = {
                name: "FRAMEBUFFER_ATTACHMENT_GREEN_SIZE",
                value: 33299,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_BLUE_SIZE = {
                name: "FRAMEBUFFER_ATTACHMENT_BLUE_SIZE",
                value: 33300,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE = {
                name: "FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE",
                value: 33301,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE = {
                name: "FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE",
                value: 33302,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE = {
                name: "FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE",
                value: 33303,
                description: " "
            }, e.FRAMEBUFFER_DEFAULT = {
                name: "FRAMEBUFFER_DEFAULT",
                value: 33304,
                description: " "
            }, e.DEPTH24_STENCIL8 = {
                name: "DEPTH24_STENCIL8",
                value: 35056,
                description: " "
            }, e.DRAW_FRAMEBUFFER_BINDING = {
                name: "DRAW_FRAMEBUFFER_BINDING",
                value: 36006,
                description: " "
            }, e.READ_FRAMEBUFFER = {
                name: "READ_FRAMEBUFFER",
                value: 36008,
                description: " "
            }, e.DRAW_FRAMEBUFFER = {
                name: "DRAW_FRAMEBUFFER",
                value: 36009,
                description: " "
            }, e.READ_FRAMEBUFFER_BINDING = {
                name: "READ_FRAMEBUFFER_BINDING",
                value: 36010,
                description: " "
            }, e.RENDERBUFFER_SAMPLES = {
                name: "RENDERBUFFER_SAMPLES",
                value: 36011,
                description: " "
            }, e.FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER = {
                name: "FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER",
                value: 36052,
                description: " "
            }, e.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE = {
                name: "FRAMEBUFFER_INCOMPLETE_MULTISAMPLE",
                value: 36182,
                description: " "
            }, e.UNIFORM_BUFFER = {
                name: "UNIFORM_BUFFER",
                value: 35345,
                description: " "
            }, e.UNIFORM_BUFFER_BINDING = {
                name: "UNIFORM_BUFFER_BINDING",
                value: 35368,
                description: " "
            }, e.UNIFORM_BUFFER_START = {
                name: "UNIFORM_BUFFER_START",
                value: 35369,
                description: " "
            }, e.UNIFORM_BUFFER_SIZE = {
                name: "UNIFORM_BUFFER_SIZE",
                value: 35370,
                description: " "
            }, e.MAX_VERTEX_UNIFORM_BLOCKS = {
                name: "MAX_VERTEX_UNIFORM_BLOCKS",
                value: 35371,
                description: " "
            }, e.MAX_FRAGMENT_UNIFORM_BLOCKS = {
                name: "MAX_FRAGMENT_UNIFORM_BLOCKS",
                value: 35373,
                description: " "
            }, e.MAX_COMBINED_UNIFORM_BLOCKS = {
                name: "MAX_COMBINED_UNIFORM_BLOCKS",
                value: 35374,
                description: " "
            }, e.MAX_UNIFORM_BUFFER_BINDINGS = {
                name: "MAX_UNIFORM_BUFFER_BINDINGS",
                value: 35375,
                description: " "
            }, e.MAX_UNIFORM_BLOCK_SIZE = {
                name: "MAX_UNIFORM_BLOCK_SIZE",
                value: 35376,
                description: " "
            }, e.MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS = {
                name: "MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS",
                value: 35377,
                description: " "
            }, e.MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS = {
                name: "MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS",
                value: 35379,
                description: " "
            }, e.UNIFORM_BUFFER_OFFSET_ALIGNMENT = {
                name: "UNIFORM_BUFFER_OFFSET_ALIGNMENT",
                value: 35380,
                description: " "
            }, e.ACTIVE_UNIFORM_BLOCKS = {
                name: "ACTIVE_UNIFORM_BLOCKS",
                value: 35382,
                description: " "
            }, e.UNIFORM_TYPE = {
                name: "UNIFORM_TYPE",
                value: 35383,
                description: " "
            }, e.UNIFORM_SIZE = {
                name: "UNIFORM_SIZE",
                value: 35384,
                description: " "
            }, e.UNIFORM_BLOCK_INDEX = {
                name: "UNIFORM_BLOCK_INDEX",
                value: 35386,
                description: " "
            }, e.UNIFORM_OFFSET = {
                name: "UNIFORM_OFFSET",
                value: 35387,
                description: " "
            }, e.UNIFORM_ARRAY_STRIDE = {
                name: "UNIFORM_ARRAY_STRIDE",
                value: 35388,
                description: " "
            }, e.UNIFORM_MATRIX_STRIDE = {
                name: "UNIFORM_MATRIX_STRIDE",
                value: 35389,
                description: " "
            }, e.UNIFORM_IS_ROW_MAJOR = {
                name: "UNIFORM_IS_ROW_MAJOR",
                value: 35390,
                description: " "
            }, e.UNIFORM_BLOCK_BINDING = {
                name: "UNIFORM_BLOCK_BINDING",
                value: 35391,
                description: " "
            }, e.UNIFORM_BLOCK_DATA_SIZE = {
                name: "UNIFORM_BLOCK_DATA_SIZE",
                value: 35392,
                description: " "
            }, e.UNIFORM_BLOCK_ACTIVE_UNIFORMS = {
                name: "UNIFORM_BLOCK_ACTIVE_UNIFORMS",
                value: 35394,
                description: " "
            }, e.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES = {
                name: "UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES",
                value: 35395,
                description: " "
            }, e.UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER = {
                name: "UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER",
                value: 35396,
                description: " "
            }, e.UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER = {
                name: "UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER",
                value: 35398,
                description: " "
            }, e.OBJECT_TYPE = {
                name: "OBJECT_TYPE",
                value: 37138,
                description: " "
            }, e.SYNC_CONDITION = {
                name: "SYNC_CONDITION",
                value: 37139,
                description: " "
            }, e.SYNC_STATUS = {
                name: "SYNC_STATUS",
                value: 37140,
                description: " "
            }, e.SYNC_FLAGS = {
                name: "SYNC_FLAGS",
                value: 37141,
                description: " "
            }, e.SYNC_FENCE = {
                name: "SYNC_FENCE",
                value: 37142,
                description: " "
            }, e.SYNC_GPU_COMMANDS_COMPLETE = {
                name: "SYNC_GPU_COMMANDS_COMPLETE",
                value: 37143,
                description: " "
            }, e.UNSIGNALED = {
                name: "UNSIGNALED",
                value: 37144,
                description: " "
            }, e.SIGNALED = {
                name: "SIGNALED",
                value: 37145,
                description: " "
            }, e.ALREADY_SIGNALED = {
                name: "ALREADY_SIGNALED",
                value: 37146,
                description: " "
            }, e.TIMEOUT_EXPIRED = {
                name: "TIMEOUT_EXPIRED",
                value: 37147,
                description: " "
            }, e.CONDITION_SATISFIED = {
                name: "CONDITION_SATISFIED",
                value: 37148,
                description: " "
            }, e.WAIT_FAILED = {
                name: "WAIT_FAILED",
                value: 37149,
                description: " "
            }, e.SYNC_FLUSH_COMMANDS_BIT = {
                name: "SYNC_FLUSH_COMMANDS_BIT",
                value: 1,
                description: " "
            }, e.COLOR = {
                name: "COLOR",
                value: 6144,
                description: " "
            }, e.DEPTH = {
                name: "DEPTH",
                value: 6145,
                description: " "
            }, e.STENCIL = {
                name: "STENCIL",
                value: 6146,
                description: " "
            }, e.MIN = {
                name: "MIN",
                value: 32775,
                description: " "
            }, e.MAX = {
                name: "MAX",
                value: 32776,
                description: " "
            }, e.DEPTH_COMPONENT24 = {
                name: "DEPTH_COMPONENT24",
                value: 33190,
                description: " "
            }, e.STREAM_READ = {
                name: "STREAM_READ",
                value: 35041,
                description: " "
            }, e.STREAM_COPY = {
                name: "STREAM_COPY",
                value: 35042,
                description: " "
            }, e.STATIC_READ = {
                name: "STATIC_READ",
                value: 35045,
                description: " "
            }, e.STATIC_COPY = {
                name: "STATIC_COPY",
                value: 35046,
                description: " "
            }, e.DYNAMIC_READ = {
                name: "DYNAMIC_READ",
                value: 35049,
                description: " "
            }, e.DYNAMIC_COPY = {
                name: "DYNAMIC_COPY",
                value: 35050,
                description: " "
            }, e.DEPTH_COMPONENT32F = {
                name: "DEPTH_COMPONENT32F",
                value: 36012,
                description: " "
            }, e.DEPTH32F_STENCIL8 = {
                name: "DEPTH32F_STENCIL8",
                value: 36013,
                description: " "
            }, e.INVALID_INDEX = {
                name: "INVALID_INDEX",
                value: 4294967295,
                description: " "
            }, e.TIMEOUT_IGNORED = {
                name: "TIMEOUT_IGNORED",
                value: -1,
                description: " "
            }, e.MAX_CLIENT_WAIT_TIMEOUT_WEBGL = {
                name: "MAX_CLIENT_WAIT_TIMEOUT_WEBGL",
                value: 37447,
                description: " "
            }, e.VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE = {
                name: "VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE",
                value: 35070,
                description: "Describes the frequency divisor used for instanced rendering.",
                extensionName: "ANGLE_instanced_arrays"
            }, e.UNMASKED_VENDOR_WEBGL = {
                name: "UNMASKED_VENDOR_WEBGL",
                value: 37445,
                description: "Passed to getParameter to get the vendor string of the graphics driver.",
                extensionName: "ANGLE_instanced_arrays"
            }, e.UNMASKED_RENDERER_WEBGL = {
                name: "UNMASKED_RENDERER_WEBGL",
                value: 37446,
                description: "Passed to getParameter to get the renderer string of the graphics driver.",
                extensionName: "WEBGL_debug_renderer_info"
            }, e.MAX_TEXTURE_MAX_ANISOTROPY_EXT = {
                name: "MAX_TEXTURE_MAX_ANISOTROPY_EXT",
                value: 34047,
                description: "Returns the maximum available anisotropy.",
                extensionName: "EXT_texture_filter_anisotropic"
            }, e.TEXTURE_MAX_ANISOTROPY_EXT = {
                name: "TEXTURE_MAX_ANISOTROPY_EXT",
                value: 34046,
                description: "Passed to texParameter to set the desired maximum anisotropy for a texture.",
                extensionName: "EXT_texture_filter_anisotropic"
            }, e.COMPRESSED_RGB_S3TC_DXT1_EXT = {
                name: "COMPRESSED_RGB_S3TC_DXT1_EXT",
                value: 33776,
                description: "A DXT1-compressed image in an RGB image format.",
                extensionName: "WEBGL_compressed_texture_s3tc"
            }, e.COMPRESSED_RGBA_S3TC_DXT1_EXT = {
                name: "COMPRESSED_RGBA_S3TC_DXT1_EXT",
                value: 33777,
                description: "A DXT1-compressed image in an RGB image format with a simple on/off alpha value.",
                extensionName: "WEBGL_compressed_texture_s3tc"
            }, e.COMPRESSED_RGBA_S3TC_DXT3_EXT = {
                name: "COMPRESSED_RGBA_S3TC_DXT3_EXT",
                value: 33778,
                description: "A DXT3-compressed image in an RGBA image format. Compared to a 32-bit RGBA texture, it offers 4:1 compression.",
                extensionName: "WEBGL_compressed_texture_s3tc"
            }, e.COMPRESSED_RGBA_S3TC_DXT5_EXT = {
                name: "COMPRESSED_RGBA_S3TC_DXT5_EXT",
                value: 33779,
                description: "A DXT5-compressed image in an RGBA image format. It also provides a 4:1 compression, but differs to the DXT3 compression in how the alpha compression is done.",
                extensionName: "WEBGL_compressed_texture_s3tc"
            }, e.COMPRESSED_R11_EAC = {
                name: "COMPRESSED_R11_EAC",
                value: 37488,
                description: "One-channel (red) unsigned format compression.",
                extensionName: "WEBGL_compressed_texture_etc"
            }, e.COMPRESSED_SIGNED_R11_EAC = {
                name: "COMPRESSED_SIGNED_R11_EAC",
                value: 37489,
                description: "One-channel (red) signed format compression.",
                extensionName: "WEBGL_compressed_texture_etc"
            }, e.COMPRESSED_RG11_EAC = {
                name: "COMPRESSED_RG11_EAC",
                value: 37490,
                description: "Two-channel (red and green) unsigned format compression.",
                extensionName: "WEBGL_compressed_texture_etc"
            }, e.COMPRESSED_SIGNED_RG11_EAC = {
                name: "COMPRESSED_SIGNED_RG11_EAC",
                value: 37491,
                description: "Two-channel (red and green) signed format compression.",
                extensionName: "WEBGL_compressed_texture_etc"
            }, e.COMPRESSED_RGB8_ETC2 = {
                name: "COMPRESSED_RGB8_ETC2",
                value: 37492,
                description: "Compresses RBG8 data with no alpha channel.",
                extensionName: "WEBGL_compressed_texture_etc"
            }, e.COMPRESSED_RGBA8_ETC2_EAC = {
                name: "COMPRESSED_RGBA8_ETC2_EAC",
                value: 37493,
                description: "Compresses RGBA8 data. The RGB part is encoded the same as RGB_ETC2, but the alpha part is encoded separately.",
                extensionName: "WEBGL_compressed_texture_etc"
            }, e.COMPRESSED_SRGB8_ETC2 = {
                name: "COMPRESSED_SRGB8_ETC2",
                value: 37494,
                description: "Compresses sRBG8 data with no alpha channel.",
                extensionName: "WEBGL_compressed_texture_etc"
            }, e.COMPRESSED_SRGB8_ALPHA8_ETC2_EAC = {
                name: "COMPRESSED_SRGB8_ALPHA8_ETC2_EAC",
                value: 37495,
                description: "Compresses sRGBA8 data. The sRGB part is encoded the same as SRGB_ETC2, but the alpha part is encoded separately.",
                extensionName: "WEBGL_compressed_texture_etc"
            }, e.COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2 = {
                name: "COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2",
                value: 37496,
                description: "Similar to RGB8_ETC, but with ability to punch through the alpha channel, which means to make it completely opaque or transparent.",
                extensionName: "WEBGL_compressed_texture_etc"
            }, e.COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2 = {
                name: "COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2",
                value: 37497,
                description: "Similar to SRGB8_ETC, but with ability to punch through the alpha channel, which means to make it completely opaque or transparent.",
                extensionName: "WEBGL_compressed_texture_etc"
            }, e.COMPRESSED_RGB_PVRTC_4BPPV1_IMG = {
                name: "COMPRESSED_RGB_PVRTC_4BPPV1_IMG",
                value: 35840,
                description: "RGB compression in 4-bit mode. One block for each 4×4 pixels.",
                extensionName: "WEBGL_compressed_texture_pvrtc"
            }, e.COMPRESSED_RGBA_PVRTC_4BPPV1_IMG = {
                name: "COMPRESSED_RGBA_PVRTC_4BPPV1_IMG",
                value: 35842,
                description: "RGBA compression in 4-bit mode. One block for each 4×4 pixels.",
                extensionName: "WEBGL_compressed_texture_pvrtc"
            }, e.COMPRESSED_RGB_PVRTC_2BPPV1_IMG = {
                name: "COMPRESSED_RGB_PVRTC_2BPPV1_IMG",
                value: 35841,
                description: "RGB compression in 2-bit mode. One block for each 8×4 pixels.",
                extensionName: "WEBGL_compressed_texture_pvrtc"
            }, e.COMPRESSED_RGBA_PVRTC_2BPPV1_IMG = {
                name: "COMPRESSED_RGBA_PVRTC_2BPPV1_IMG",
                value: 35843,
                description: "RGBA compression in 2-bit mode. One block for each 8×4 pixe",
                extensionName: "WEBGL_compressed_texture_pvrtc"
            }, e.COMPRESSED_RGB_ETC1_WEBGL = {
                name: "COMPRESSED_RGB_ETC1_WEBGL",
                value: 36196,
                description: "Compresses 24-bit RGB data with no alpha channel.",
                extensionName: "WEBGL_compressed_texture_etc1"
            }, e.COMPRESSED_RGB_ATC_WEBGL = {
                name: "COMPRESSED_RGB_ATC_WEBGL",
                value: 35986,
                description: "Compresses RGB textures with no alpha channel.",
                extensionName: "WEBGL_compressed_texture_atc"
            }, e.COMPRESSED_RGBA_ATC_EXPLICIT_ALPHA_WEBGL = {
                name: "COMPRESSED_RGBA_ATC_EXPLICIT_ALPHA_WEBGL",
                value: 35986,
                description: "Compresses RGBA textures using explicit alpha encoding (useful when alpha transitions are sharp).",
                extensionName: "WEBGL_compressed_texture_atc"
            }, e.COMPRESSED_RGBA_ATC_INTERPOLATED_ALPHA_WEBGL = {
                name: "COMPRESSED_RGBA_ATC_INTERPOLATED_ALPHA_WEBGL",
                value: 34798,
                description: "Compresses RGBA textures using interpolated alpha encoding (useful when alpha transitions are gradient).",
                extensionName: "WEBGL_compressed_texture_atc"
            }, e.UNSIGNED_INT_24_8_WEBGL = {
                name: "UNSIGNED_INT_24_8_WEBGL",
                value: 34042,
                description: "Unsigned integer type for 24-bit depth texture data.",
                extensionName: "WEBGL_depth_texture"
            }, e.HALF_FLOAT_OES = {
                name: "HALF_FLOAT_OES",
                value: 36193,
                description: "Half floating-point type (16-bit).",
                extensionName: "OES_texture_half_float"
            }, e.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT = {
                name: "FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT",
                value: 33297,
                description: " ",
                extensionName: "WEBGL_color_buffer_float"
            }, e.UNSIGNED_NORMALIZED_EXT = {
                name: "UNSIGNED_NORMALIZED_EXT",
                value: 35863,
                description: " ",
                extensionName: "WEBGL_color_buffer_float"
            }, e.MIN_EXT = {
                name: "MIN_EXT",
                value: 32775,
                description: "Produces the minimum color components of the source and destination colors.",
                extensionName: "EXT_blend_minmax"
            }, e.MAX_EXT = {
                name: "MAX_EXT",
                value: 32776,
                description: "Produces the maximum color components of the source and destination colors.",
                extensionName: "EXT_blend_minmax"
            }, e.SRGB_EXT = {
                name: "SRGB_EXT",
                value: 35904,
                description: "Unsized sRGB format that leaves the precision up to the driver.",
                extensionName: "EXT_sRGB"
            }, e.SRGB_ALPHA_EXT = {
                name: "SRGB_ALPHA_EXT",
                value: 35906,
                description: "Unsized sRGB format with unsized alpha component.",
                extensionName: "EXT_sRGB"
            }, e.SRGB8_ALPHA8_EXT = {
                name: "SRGB8_ALPHA8_EXT",
                value: 35907,
                description: "Sized (8-bit) sRGB and alpha formats.",
                extensionName: "EXT_sRGB"
            }, e.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING_EXT = {
                name: "FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING_EXT",
                value: 33296,
                description: "Returns the framebuffer color encoding.",
                extensionName: "EXT_sRGB"
            }, e.FRAGMENT_SHADER_DERIVATIVE_HINT_OES = {
                name: "FRAGMENT_SHADER_DERIVATIVE_HINT_OES",
                value: 35723,
                description: "Indicates the accuracy of the derivative calculation for the GLSL built-in functions: dFdx, dFdy, and fwidth.",
                extensionName: "OES_standard_derivatives"
            }, e.COLOR_ATTACHMENT0_WEBGL = {
                name: "COLOR_ATTACHMENT0_WEBGL",
                value: 36064,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT1_WEBGL = {
                name: "COLOR_ATTACHMENT1_WEBGL",
                value: 36065,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT2_WEBGL = {
                name: "COLOR_ATTACHMENT2_WEBGL",
                value: 36066,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT3_WEBGL = {
                name: "COLOR_ATTACHMENT3_WEBGL",
                value: 36067,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT4_WEBGL = {
                name: "COLOR_ATTACHMENT4_WEBGL",
                value: 36068,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT5_WEBGL = {
                name: "COLOR_ATTACHMENT5_WEBGL",
                value: 36069,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT6_WEBGL = {
                name: "COLOR_ATTACHMENT6_WEBGL",
                value: 36070,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT7_WEBGL = {
                name: "COLOR_ATTACHMENT7_WEBGL",
                value: 36071,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT8_WEBGL = {
                name: "COLOR_ATTACHMENT8_WEBGL",
                value: 36072,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT9_WEBGL = {
                name: "COLOR_ATTACHMENT9_WEBGL",
                value: 36073,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT10_WEBGL = {
                name: "COLOR_ATTACHMENT10_WEBGL",
                value: 36074,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT11_WEBGL = {
                name: "COLOR_ATTACHMENT11_WEBGL",
                value: 36075,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT12_WEBGL = {
                name: "COLOR_ATTACHMENT12_WEBGL",
                value: 36076,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT13_WEBGL = {
                name: "COLOR_ATTACHMENT13_WEBGL",
                value: 36077,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT14_WEBGL = {
                name: "COLOR_ATTACHMENT14_WEBGL",
                value: 36078,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.COLOR_ATTACHMENT15_WEBGL = {
                name: "COLOR_ATTACHMENT15_WEBGL",
                value: 36079,
                description: "Framebuffer color attachment point",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER0_WEBGL = {
                name: "DRAW_BUFFER0_WEBGL",
                value: 34853,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER1_WEBGL = {
                name: "DRAW_BUFFER1_WEBGL",
                value: 34854,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER2_WEBGL = {
                name: "DRAW_BUFFER2_WEBGL",
                value: 34855,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER3_WEBGL = {
                name: "DRAW_BUFFER3_WEBGL",
                value: 34856,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER4_WEBGL = {
                name: "DRAW_BUFFER4_WEBGL",
                value: 34857,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER5_WEBGL = {
                name: "DRAW_BUFFER5_WEBGL",
                value: 34858,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER6_WEBGL = {
                name: "DRAW_BUFFER6_WEBGL",
                value: 34859,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER7_WEBGL = {
                name: "DRAW_BUFFER7_WEBGL",
                value: 34860,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER8_WEBGL = {
                name: "DRAW_BUFFER8_WEBGL",
                value: 34861,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER9_WEBGL = {
                name: "DRAW_BUFFER9_WEBGL",
                value: 34862,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER10_WEBGL = {
                name: "DRAW_BUFFER10_WEBGL",
                value: 34863,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER11_WEBGL = {
                name: "DRAW_BUFFER11_WEBGL",
                value: 34864,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER12_WEBGL = {
                name: "DRAW_BUFFER12_WEBGL",
                value: 34865,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER13_WEBGL = {
                name: "DRAW_BUFFER13_WEBGL",
                value: 34866,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER14_WEBGL = {
                name: "DRAW_BUFFER14_WEBGL",
                value: 34867,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.DRAW_BUFFER15_WEBGL = {
                name: "DRAW_BUFFER15_WEBGL",
                value: 34868,
                description: "Draw buffer",
                extensionName: "WEBGL_draw_buffers"
            }, e.MAX_COLOR_ATTACHMENTS_WEBGL = {
                name: "MAX_COLOR_ATTACHMENTS_WEBGL",
                value: 36063,
                description: "Maximum number of framebuffer color attachment points",
                extensionName: "WEBGL_draw_buffers"
            }, e.MAX_DRAW_BUFFERS_WEBGL = {
                name: "MAX_DRAW_BUFFERS_WEBGL",
                value: 34852,
                description: "Maximum number of draw buffers",
                extensionName: "WEBGL_draw_buffers"
            }, e.VERTEX_ARRAY_BINDING_OES = {
                name: "VERTEX_ARRAY_BINDING_OES",
                value: 34229,
                description: "The bound vertex array object (VAO).",
                extensionName: "VERTEX_ARRAY_BINDING_OES"
            }, e.QUERY_COUNTER_BITS_EXT = {
                name: "QUERY_COUNTER_BITS_EXT",
                value: 34916,
                description: "The number of bits used to hold the query result for the given target.",
                extensionName: "EXT_disjoint_timer_query"
            }, e.CURRENT_QUERY_EXT = {
                name: "CURRENT_QUERY_EXT",
                value: 34917,
                description: "The currently active query.",
                extensionName: "EXT_disjoint_timer_query"
            }, e.QUERY_RESULT_EXT = {
                name: "QUERY_RESULT_EXT",
                value: 34918,
                description: "The query result.",
                extensionName: "EXT_disjoint_timer_query"
            }, e.QUERY_RESULT_AVAILABLE_EXT = {
                name: "QUERY_RESULT_AVAILABLE_EXT",
                value: 34919,
                description: "A Boolean indicating whether or not a query result is available.",
                extensionName: "EXT_disjoint_timer_query"
            }, e.TIME_ELAPSED_EXT = {
                name: "TIME_ELAPSED_EXT",
                value: 35007,
                description: "Elapsed time (in nanoseconds).",
                extensionName: "EXT_disjoint_timer_query"
            }, e.TIMESTAMP_EXT = {
                name: "TIMESTAMP_EXT",
                value: 36392,
                description: "The current time.",
                extensionName: "EXT_disjoint_timer_query"
            }, e.GPU_DISJOINT_EXT = {
                name: "GPU_DISJOINT_EXT",
                value: 36795,
                description: "A Boolean indicating whether or not the GPU performed any disjoint operation.",
                extensionName: "EXT_disjoint_timer_query"
            }, e.zeroMeaningByCommand = {
                getError: "NO_ERROR",
                blendFunc: "ZERO",
                blendFuncSeparate: "ZERO",
                readBuffer: "NONE",
                getFramebufferAttachmentParameter: "NONE",
                texParameterf: "NONE",
                texParameteri: "NONE",
                drawArrays: "POINTS",
                drawElements: "POINTS",
                drawArraysInstanced: "POINTS",
                drawArraysInstancedAngle: "POINTS",
                drawBuffers: "POINTS",
                drawElementsInstanced: "POINTS",
                drawRangeElements: "POINTS"
            }, e.oneMeaningByCommand = {
                blendFunc: "ONE",
                blendFuncSeparate: "ONE",
                drawArrays: "LINES",
                drawElements: "LINES",
                drawArraysInstanced: "LINES",
                drawArraysInstancedAngle: "LINES",
                drawBuffers: "LINES",
                drawElementsInstanced: "LINES",
                drawRangeElements: "LINES"
            }, e
        }(),
        T = {},
        R = {};
    ! function () {
        for (var e in m)
            if (m.hasOwnProperty(e)) {
                var t = m[e];
                T[t.name] = t, R[t.value] = t
            }
    }();
    var f = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        A = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return f(t, e), Object.defineProperty(t.prototype, "analyserName", {
                get: function () {
                    return t.analyserName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.appendToAnalysis = function (e, t) {
                if (e.commands) {
                    for (var n = {
                            total: 0,
                            totalTriangles: 0,
                            totalTriangleStrip: 0,
                            totalTriangleFan: 0,
                            totalLines: 0,
                            totalLineStrip: 0,
                            totalLineLoop: 0,
                            totalPoints: 0
                        }, r = 0, a = e.commands; r < a.length; r++) {
                        var o = a[r];
                        "drawArrays" === o.name && o.commandArguments.length >= 3 || "drawArraysInstanced" === o.name && o.commandArguments.length >= 3 || "drawArraysInstancedANGLE" === o.name && o.commandArguments.length >= 3 ? this.appendToPrimitives(n, o.commandArguments[0], o.commandArguments[2]) : "drawElements" === o.name && o.commandArguments.length >= 2 || "drawElementsInstanced" === o.name && o.commandArguments.length >= 2 || "drawElementsInstancedANGLE" === o.name && o.commandArguments.length >= 2 ? this.appendToPrimitives(n, o.commandArguments[0], o.commandArguments[1]) : "drawRangeElements" === o.name && o.commandArguments.length >= 4 && this.appendToPrimitives(n, o.commandArguments[0], o.commandArguments[3])
                    }
                    t.total = n.total, t.triangles = n.totalTriangles, t.triangleStrip = n.totalTriangleStrip, t.triangleFan = n.totalTriangleFan, t.lines = n.totalLines, t.lineStrip = n.totalLineStrip, t.lineLoop = n.totalLineLoop, t.points = n.totalPoints
                }
            }, t.prototype.appendToPrimitives = function (e, t, n) {
                t === m.POINTS.value ? e.totalPoints += n : t === m.LINES.value ? e.totalLines += n : t === m.LINE_STRIP.value ? e.totalLineStrip += n : t === m.LINE_LOOP.value ? e.totalLineLoop += n : t === m.TRIANGLES.value ? e.totalTriangles += n : t === m.TRIANGLE_STRIP.value ? e.totalTriangleStrip += n : t === m.TRIANGLE_FAN.value && (e.totalTriangleFan += n), e.total += n
            }, t.analyserName = "Primitives", t
        }(u),
        d = function () {
            function e(e) {
                this.contextInformation = e, this.analysers = [], this.initAnalysers()
            }
            return e.prototype.appendAnalyses = function (e) {
                for (var t in this.analysers) this.analysers.hasOwnProperty(t) && this.analysers[t].appendAnalysis(e)
            }, e.prototype.initAnalysers = function () {
                this.analysers.push(new E(this.contextInformation), new l(this.contextInformation), new A(this.contextInformation))
            }, e
        }(),
        h = function () {
            function e() {}
            return e.storeOriginFunction = function (e, t) {
                if (e && e[t]) {
                    var n = this.getOriginFunctionName(t);
                    e[n] || (e[n] = e[t])
                }
            }, e.storePrototypeOriginFunction = function (e, t) {
                if (e && e.prototype[t]) {
                    var n = this.getOriginFunctionName(t);
                    e.prototype[n] || (e.prototype[n] = e.prototype[t])
                }
            }, e.executePrototypeOriginFunction = function (e, t, n, r) {
                if (e) {
                    var a = this.getOriginFunctionName(n);
                    if (t.prototype[a]) return e[a] || (e[a] = t.prototype[a]), this.executeFunction(e, a, r)
                }
            }, e.executeOriginFunction = function (e, t, n) {
                if (e) {
                    var r = this.getOriginFunctionName(t);
                    if (e[r]) return this.executeFunction(e, r, n)
                }
            }, e.executeFunction = function (e, t, n) {
                var r = n;
                if (void 0 === r || 0 === r.length) return e[t]();
                switch (r.length) {
                    case 1:
                        return e[t](r[0]);
                    case 2:
                        return e[t](r[0], r[1]);
                    case 3:
                        return e[t](r[0], r[1], r[2]);
                    case 4:
                        return e[t](r[0], r[1], r[2], r[3]);
                    case 5:
                        return e[t](r[0], r[1], r[2], r[3], r[4]);
                    case 6:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5]);
                    case 7:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6]);
                    case 8:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]);
                    case 9:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8]);
                    case 10:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9]);
                    case 11:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10]);
                    case 12:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10], r[11]);
                    case 13:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10], r[11], r[12]);
                    case 14:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13]);
                    case 15:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13], r[14]);
                    case 16:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13], r[14], r[15]);
                    case 17:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13], r[14], r[15], r[16]);
                    case 18:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13], r[14], r[15], r[16], r[17]);
                    case 19:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13], r[14], r[15], r[16], r[17], r[18]);
                    case 20:
                        return e[t](r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9], r[10], r[11], r[12], r[13], r[14], r[15], r[16], r[17], r[18], r[19]);
                    default:
                        return e[t].apply(e, r)
                }
            }, e.getOriginFunctionName = function (e) {
                return this.originFunctionPrefix + e
            }, e.originFunctionPrefix = "__SPECTOR_Origin_", e
        }(),
        C = function () {
            function e() {}
            return e.getStackTrace = function (e, t) {
                void 0 === e && (e = 0), void 0 === t && (t = 0);
                var n = [];
                try {
                    throw new Error("Errorator.")
                } catch (e) {
                    if (e.stack)
                        for (var r = 0, a = (o = e.stack.split("\n")).length; r < a; r++) o[r].match(/^\s*[A-Za-z0-9\-_\$]+\(/) ? n.push(o[r]) : 0 === o[r].indexOf("    at ") ? (o[r] = o[r].replace("    at ", ""), n.push(o[r])) : -1 !== o[r].indexOf("/<@http") ? (o[r] = o[r].substring(o[r].indexOf("/<@http") + 3), n.push(o[r])) : -1 !== o[r].indexOf("@http") && (o[r] = o[r].replace("@http", " (http"), o[r] = o[r] + ")", n.push(o[r]));
                    else if (e.message) {
                        var o;
                        for (r = 0, a = (o = e.message.split("\n")).length; r < a; r++)
                            if (o[r].match(/^\s*[A-Za-z0-9\-_\$]+\(/)) {
                                var i = o[r];
                                o[r + 1] && (i += " at " + o[r + 1], r++), n.push(i)
                            }
                    }
                }
                if (!n)
                    for (var s = arguments.callee.caller; s;) {
                        var u = s.toString(),
                            c = u.substring(u.indexOf("function") + 8, u.indexOf("")) || "anonymous";
                        n.push(c), s = s.caller
                    }
                if (n) {
                    for (n.shift(), r = 0; r < e && n.length > 0; r++) n.shift();
                    for (r = 0; r < t && n.length > 0; r++) n.pop()
                }
                return n
            }, e
        }(),
        N = function () {
            function e() {}
            return e.getWebGlObjectTag = function (t) {
                return t[e.SPECTOROBJECTTAGKEY]
            }, e.attachWebGlObjectTag = function (t, n) {
                n.displayText = e.stringifyWebGlObjectTag(n), t[e.SPECTOROBJECTTAGKEY] = n
            }, e.stringifyWebGlObjectTag = function (e) {
                return e ? "".concat(e.typeName, " - ID: ").concat(e.id) : "No tag available."
            }, e.SPECTOROBJECTTAGKEY = "__SPECTOR_Object_TAG", e
        }(),
        S = function () {
            function e() {
                this.id = 0
            }
            return Object.defineProperty(e.prototype, "type", {
                get: function () {
                    return window[this.typeName] || null
                },
                enumerable: !1,
                configurable: !0
            }), e.prototype.tagWebGlObject = function (e) {
                if (this.type) {
                    var t;
                    if (!e) return t;
                    if (t = N.getWebGlObjectTag(e)) return t;
                    if (e instanceof this.type) {
                        var n = this.getNextId();
                        return t = {
                            typeName: this.typeName,
                            id: n
                        }, N.attachWebGlObjectTag(e, t), t
                    }
                    return t
                }
            }, e.prototype.getNextId = function () {
                return this.id++
            }, e
        }(),
        v = function () {
            function e(e) {
                this.options = e
            }
            return e.prototype.createCapture = function (e, t, n) {
                var r = C.getStackTrace(4, 1),
                    a = 0 === e.name.indexOf("uniform") ? this.stringifyUniform(e.arguments) : this.stringify(e.arguments, e.result),
                    o = {
                        id: t,
                        startTime: e.startTime,
                        commandEndTime: e.endTime,
                        endTime: 0,
                        name: e.name,
                        commandArguments: e.arguments,
                        result: e.result,
                        stackTrace: r,
                        status: 0,
                        marker: n,
                        text: a
                    };
                this.transformCapture(o);
                for (var i = 0; i < o.commandArguments.length; i++) {
                    var s = o.commandArguments[i];
                    s && s.length && s.length > 50 && (o.commandArguments[i] = "Array Length: " + s.length)
                }
                if (o.commandArguments) {
                    var u = [];
                    for (i = 0; i < o.commandArguments.length; i++) {
                        var c = o.commandArguments[i];
                        void 0 === c ? u.push(void 0) : null === c ? u.push(null) : u.push(JSON.parse(this.stringifyJSON(c)))
                    }
                    o.commandArguments = u
                }
                return o.result && (o.result = JSON.parse(this.stringifyJSON(o.result))), o
            }, e.prototype.stringifyJSON = function (e) {
                try {
                    return JSON.stringify(e)
                } catch (e) {
                    return null
                }
            }, e.prototype.transformCapture = function (e) {}, e.prototype.stringify = function (e, t) {
                var n = this.spiedCommandName;
                return e && e.length > 0 && (n += ": " + this.stringifyArgs(e).join(", ")), t && (n += " -> " + this.stringifyResult(t)), n
            }, e.prototype.stringifyUniform = function (e) {
                var t = this.spiedCommandName;
                if (e && e.length > 0) {
                    var n = [];
                    n.push(this.stringifyValue(e[0]));
                    for (var r = 1; r < e.length; r++)
                        if ("number" == typeof e[r]) {
                            var a = e[r] + "";
                            n.push(a)
                        } else a = this.stringifyValue(e[r]), n.push(a);
                    t += ": " + n.join(", ")
                }
                return t
            }, e.prototype.stringifyArgs = function (e) {
                for (var t = [], n = 0; n < e.length; n++) {
                    var r = e[n],
                        a = this.stringifyValue(r);
                    t.push(a)
                }
                return t
            }, e.prototype.stringifyResult = function (e) {
                if (e) return this.stringifyValue(e)
            }, e.prototype.stringifyValue = function (e) {
                if (null === e) return "null";
                if (void 0 === e) return "undefined";
                var t = N.getWebGlObjectTag(e);
                return t ? t.displayText : "number" == typeof e && m.isWebGlConstant(e) ? m.stringifyWebGlConstant(e, this.spiedCommandName) : "string" == typeof e ? e : e instanceof HTMLImageElement ? e.src : e instanceof ArrayBuffer ? "[--(" + e.byteLength + ")--]" : e.length ? "[..(" + e.length + ")..]" : e
            }, e
        }(),
        O = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        F = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return O(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                if (e.length > 0) {
                    var n = e[0],
                        r = this.stringifyValue(n);
                    t.push(r)
                }
                return e.length > 1 && (n = "" + e[1], t.push(n)), e.length > 2 && t.push(e[2]), t
            }, t.commandName = "bindAttribLocation", t
        }(v),
        g = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        y = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return g(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [],
                    n = this.options.context.getParameter(m.READ_FRAMEBUFFER_BINDING.value),
                    r = this.options.tagWebGlObject(n);
                t.push("READ FROM: " + this.stringifyValue(r));
                var a = this.options.context.getParameter(m.DRAW_FRAMEBUFFER_BINDING.value),
                    o = this.options.tagWebGlObject(a);
                t.push("WRITE TO: " + this.stringifyValue(o));
                for (var i = 0; i < 8; i++) t.push(e[i]);
                return (e[8] & m.DEPTH_BUFFER_BIT.value) === m.DEPTH_BUFFER_BIT.value && t.push(m.DEPTH_BUFFER_BIT.name), (e[8] & m.STENCIL_BUFFER_BIT.value) === m.STENCIL_BUFFER_BIT.value && t.push(m.STENCIL_BUFFER_BIT.name), (e[8] & m.COLOR_BUFFER_BIT.value) === m.COLOR_BUFFER_BIT.value && t.push(m.COLOR_BUFFER_BIT.name), t.push(m.stringifyWebGlConstant(e[9], "blitFrameBuffer")), t
            }, t.commandName = "blitFrameBuffer", t
        }(v),
        I = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        B = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return I(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return (e[0] & m.DEPTH_BUFFER_BIT.value) === m.DEPTH_BUFFER_BIT.value && t.push(m.DEPTH_BUFFER_BIT.name), (e[0] & m.STENCIL_BUFFER_BIT.value) === m.STENCIL_BUFFER_BIT.value && t.push(m.STENCIL_BUFFER_BIT.name), (e[0] & m.COLOR_BUFFER_BIT.value) === m.COLOR_BUFFER_BIT.value && t.push(m.COLOR_BUFFER_BIT.name), t
            }, t.commandName = "clear", t
        }(v),
        P = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        M = ["lineWidth"],
        L = function (e) {
            function t(t, n) {
                var r = e.call(this, t) || this;
                return r.internalSpiedCommandName = n, r.isDeprecated = M.indexOf(r.spiedCommandName) > -1, r
            }
            return P(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return this.internalSpiedCommandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.transformCapture = function (e) {
                this.isDeprecated && (e.status = 50)
            }, t
        }(v),
        b = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        U = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return b(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(e[0]), t
            }, t.commandName = "disableVertexAttribArray", t
        }(v),
        D = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        G = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return D(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "drawArrays")), t.push(e[1]), t.push(e[2]), t
            }, t.commandName = "drawArrays", t
        }(v),
        x = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        w = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return x(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "drawArraysInstanced")), t.push(e[1]), t.push(e[2]), t.push(e[3]), t
            }, t.commandName = "drawArraysInstanced", t
        }(v),
        X = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        W = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return X(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "drawArraysInstancedANGLE")), t.push(e[1]), t.push(e[2]), t.push(e[3]), t
            }, t.commandName = "drawArraysInstancedANGLE", t
        }(v),
        V = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        H = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return V(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "drawElements")), t.push(e[1]), t.push(m.stringifyWebGlConstant(e[2], "drawElements")), t.push(e[3]), t
            }, t.commandName = "drawElements", t
        }(v),
        j = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Y = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return j(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "drawElementsInstancedANGLE")), t.push(e[1]), t.push(m.stringifyWebGlConstant(e[2], "drawElementsInstancedANGLE")), t.push(e[3]), t.push(e[4]), t
            }, t.commandName = "drawElementsInstancedANGLE", t
        }(v),
        K = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        k = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return K(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "drawElementsInstanced")), t.push(e[1]), t.push(m.stringifyWebGlConstant(e[2], "drawElementsInstanced")), t.push(e[3]), t.push(e[4]), t
            }, t.commandName = "drawElementsInstanced", t
        }(v),
        Z = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        z = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Z(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "drawRangeElements")), t.push(e[1]), t.push(e[2]), t.push(e[3]), t.push(m.stringifyWebGlConstant(e[4], "drawRangeElements")), t.push(e[5]), t
            }, t.commandName = "drawRangeElements", t
        }(v),
        q = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Q = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return q(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyResult = function (e) {
                if (e) return "name: ".concat(e.name, ", size: ").concat(e.size, ", type: ").concat(e.type)
            }, t.commandName = "getActiveAttrib", t
        }(v),
        J = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        $ = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return J(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyResult = function (e) {
                if (e) return "name: ".concat(e.name, ", size: ").concat(e.size, ", type: ").concat(e.type)
            }, t.commandName = "getActiveUniform", t
        }(v),
        ee = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        te = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return ee(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyResult = function (e) {
                return e ? "true" : "false"
            }, t.commandName = "getExtension", t
        }(v),
        ne = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        re = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return ne(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyResult = function (e) {
                if (!e) return "null";
                var t = N.getWebGlObjectTag(e);
                return t ? t.displayText : e
            }, t.commandName = "getParameter", t
        }(v),
        ae = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        oe = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return ae(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyResult = function (e) {
                if (e) return "min: ".concat(e.rangeMin, ", max: ").concat(e.rangeMax, ", precision: ").concat(e.precision)
            }, t.commandName = "getShaderPrecisionFormat", t
        }(v),
        ie = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        se = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return ie(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyResult = function (e) {
                if (e) return "name: ".concat(e.name, ", size: ").concat(e.size, ", type: ").concat(e.type)
            }, t.commandName = "getTransformFeedbackVarying", t
        }(v),
        ue = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        ce = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return ue(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "multiDrawArraysInstancedBaseInstanceWEBGL")), t.push("drawCount=".concat(e[9])), t.push(e[2]), t.push(e[4]), t.push(e[6]), t.push(e[8]), t
            }, t.commandName = "multiDrawArraysInstancedBaseInstanceWEBGL", t
        }(v),
        Ee = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        _e = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Ee(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "multiDrawElementsInstancedBaseVertexBaseInstanceWEBGL")), t.push(m.stringifyWebGlConstant(e[3], "multiDrawElementsInstancedBaseVertexBaseInstanceWEBGL")), t.push("drawCount=".concat(e[11])), t.push(e[2]), t.push(e[4]), t.push(e[6]), t.push(e[8]), t.push(e[10]), t
            }, t.commandName = "multiDrawElementsInstancedBaseVertexBaseInstanceWEBGL", t
        }(v),
        pe = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        le = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pe(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                for (var t = [], n = 0; n < 4; n++) t.push(e[n].toFixed(0));
                return t
            }, t.commandName = "scissor", t
        }(v);

    function me(e) {
        return null == e ? "" : "".concat(e.toFixed(0), " (0b").concat((e >>> 0).toString(2), ")")
    }
    var Te, Re, fe = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Ae = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return fe(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(me(e[0])), t
            }, t.commandName = "stencilMask", t
        }(v),
        de = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        he = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return de(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "stencilMaskSeparate")), t.push(me(e[1])), t
            }, t.commandName = "stencilMaskSeparate", t
        }(v),
        Ce = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Ne = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Ce(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "stencilFunc")), t.push(me(e[1])), t.push(me(e[2])), t
            }, t.commandName = "stencilFunc", t
        }(v),
        Se = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        ve = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Se(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(m.stringifyWebGlConstant(e[0], "stencilFuncSeparate")), t.push(m.stringifyWebGlConstant(e[1], "stencilFuncSeparate")), t.push(me(e[2])), t.push(me(e[3])), t
            }, t.commandName = "stencilFuncSeparate", t
        }(v),
        Oe = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Fe = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Oe(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(e[0]), t.push(e[1]), t.push(m.stringifyWebGlConstant(e[2], "vertexAttribPointer")), t.push(e[3]), t.push(e[4]), t.push(e[5]), t
            }, t.commandName = "vertexAttribPointer", t
        }(v),
        ge = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        ye = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return ge(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                for (var t = [], n = 0; n < 4; n++) t.push(e[n].toFixed(0));
                return t
            }, t.commandName = "viewport", t
        }(v),
        Ie = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Be = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Ie(t, e), Object.defineProperty(t.prototype, "spiedCommandName", {
                get: function () {
                    return t.commandName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.stringifyArgs = function (e) {
                var t = [];
                return t.push(e[0]), t
            }, t.commandName = "enableVertexAttribArray", t
        }(v),
        Pe = function () {
            function e(e) {
                this.spiedCommandName = e.spiedCommandName, this.spiedCommandRunningContext = e.spiedCommandRunningContext, this.spiedCommand = this.spiedCommandRunningContext[this.spiedCommandName], h.storeOriginFunction(this.spiedCommandRunningContext, this.spiedCommandName), this.callback = e.callback, this.commandOptions = {
                    context: e.context,
                    contextVersion: e.contextVersion,
                    extensions: e.extensions,
                    toggleCapture: e.toggleCapture
                }, this.initCustomCommands(), this.initCommand()
            }
            return e.prototype.spy = function () {
                this.spiedCommandRunningContext[this.spiedCommandName] = this.overloadedCommand
            }, e.prototype.unSpy = function () {
                this.spiedCommandRunningContext[this.spiedCommandName] = this.spiedCommand
            }, e.prototype.createCapture = function (e, t, n) {
                return this.command.createCapture(e, t, n)
            }, e.prototype.initCommand = function () {
                e.customCommandsConstructors[this.spiedCommandName] ? this.command = e.customCommandsConstructors[this.spiedCommandName](this.commandOptions) : this.command = new L(this.commandOptions, this.spiedCommandName), this.overloadedCommand = this.getSpy()
            }, e.prototype.getSpy = function () {
                var e = this;
                return function () {
                    var t = s.now,
                        n = h.executeOriginFunction(e.spiedCommandRunningContext, e.spiedCommandName, arguments),
                        r = s.now,
                        a = {
                            name: e.spiedCommandName,
                            arguments,
                            result: n,
                            startTime: t,
                            endTime: r
                        };
                    return e.callback(e, a), n
                }
            }, e.prototype.initCustomCommands = function () {
                var t;
                e.customCommandsConstructors || (e.customCommandsConstructors = ((t = {})[F.commandName] = function (e) {
                    return new F(e)
                }, t[y.commandName] = function (e) {
                    return new y(e)
                }, t[B.commandName] = function (e) {
                    return new B(e)
                }, t[U.commandName] = function (e) {
                    return new U(e)
                }, t[G.commandName] = function (e) {
                    return new G(e)
                }, t[w.commandName] = function (e) {
                    return new w(e)
                }, t[W.commandName] = function (e) {
                    return new W(e)
                }, t[H.commandName] = function (e) {
                    return new H(e)
                }, t[k.commandName] = function (e) {
                    return new k(e)
                }, t[Y.commandName] = function (e) {
                    return new Y(e)
                }, t[z.commandName] = function (e) {
                    return new z(e)
                }, t[Q.commandName] = function (e) {
                    return new Q(e)
                }, t[$.commandName] = function (e) {
                    return new $(e)
                }, t[te.commandName] = function (e) {
                    return new te(e)
                }, t[re.commandName] = function (e) {
                    return new re(e)
                }, t[oe.commandName] = function (e) {
                    return new oe(e)
                }, t[se.commandName] = function (e) {
                    return new se(e)
                }, t[ce.commandName] = function (e) {
                    return new ce(e)
                }, t[_e.commandName] = function (e) {
                    return new _e(e)
                }, t[le.commandName] = function (e) {
                    return new le(e)
                }, t[Ae.commandName] = function (e) {
                    return new Ae(e)
                }, t[he.commandName] = function (e) {
                    return new he(e)
                }, t[Ne.commandName] = function (e) {
                    return new Ne(e)
                }, t[ve.commandName] = function (e) {
                    return new ve(e)
                }, t[Fe.commandName] = function (e) {
                    return new Fe(e)
                }, t[ye.commandName] = function (e) {
                    return new ye(e)
                }, t[Be.commandName] = function (e) {
                    return new Be(e)
                }, t))
            }, e
        }(),
        Me = function () {
            function e(e) {
                this.options = e, this.context = e.context, this.contextVersion = e.contextVersion, this.extensions = e.extensions, this.toggleCapture = e.toggleCapture, this.consumeCommands = this.getConsumeCommands(), this.changeCommandsByState = this.getChangeCommandsByState(), this.commandNameToStates = this.getCommandNameToStates()
            }
            return Object.defineProperty(e.prototype, "requireStartAndStopStates", {
                get: function () {
                    return !0
                },
                enumerable: !1,
                configurable: !0
            }), e.prototype.startCapture = function (e, t, n) {
                return this.quickCapture = t, this.fullCapture = n, this.capturedCommandsByState = {}, e && this.requireStartAndStopStates && (this.currentState = {}, this.readFromContextNoSideEffects()), this.copyCurrentStateToPrevious(), this.currentState = {}, this.previousState
            }, e.prototype.stopCapture = function () {
                return this.requireStartAndStopStates && this.readFromContextNoSideEffects(), this.analyse(void 0), this.currentState
            }, e.prototype.registerCallbacks = function (e) {
                for (var t in this.changeCommandsByState)
                    if (this.changeCommandsByState.hasOwnProperty(t))
                        for (var n = 0, r = this.changeCommandsByState[t]; n < r.length; n++) {
                            var a = r[n];
                            e[a] = e[a] || [], e[a].push(this.onChangeCommand.bind(this))
                        }
                for (var o = 0, i = this.consumeCommands; o < i.length; o++) {
                    var s = i[o];
                    e[s] = e[s] || [], e[s].push(this.onConsumeCommand.bind(this))
                }
            }, e.prototype.getStateData = function () {
                return this.currentState
            }, e.prototype.getConsumeCommands = function () {
                return []
            }, e.prototype.getChangeCommandsByState = function () {
                return {}
            }, e.prototype.copyCurrentStateToPrevious = function () {
                this.currentState && (this.previousState = this.currentState)
            }, e.prototype.onChangeCommand = function (e) {
                for (var t = 0, n = this.commandNameToStates[e.name]; t < n.length; t++) {
                    var r = n[t];
                    if (!this.isValidChangeCommand(e, r)) return;
                    this.capturedCommandsByState[r] = this.capturedCommandsByState[r] || [], this.capturedCommandsByState[r].push(e)
                }
            }, e.prototype.isValidChangeCommand = function (e, t) {
                return !0
            }, e.prototype.onConsumeCommand = function (e) {
                this.isValidConsumeCommand(e) && (this.readFromContextNoSideEffects(), this.analyse(e), this.storeCommandIds(), e[this.stateName] = this.currentState, this.startCapture(!1, this.quickCapture, this.fullCapture))
            }, e.prototype.isValidConsumeCommand = function (e) {
                return this.lastCommandName = null == e ? void 0 : e.name, !0
            }, e.prototype.analyse = function (e) {
                for (var t in this.capturedCommandsByState)
                    if (this.capturedCommandsByState.hasOwnProperty(t)) {
                        var n = this.capturedCommandsByState[t],
                            r = n.length - 1;
                        if (r >= 0)
                            if (e) {
                                for (var a = 0; a < r; a++) {
                                    var o = n[a];
                                    o.consumeCommandId = e.id, this.changeCommandCaptureStatus(o, 30)
                                }
                                var i = this.isStateEnableNoSideEffects(t, e.commandArguments);
                                (s = n[r]).consumeCommandId = e.id, this.areStatesEquals(this.currentState[t], this.previousState[t]) ? this.changeCommandCaptureStatus(s, 30) : i ? this.changeCommandCaptureStatus(s, 40) : this.changeCommandCaptureStatus(s, 20)
                            } else
                                for (a = 0; a < n.length; a++) {
                                    var s = n[a];
                                    this.changeCommandCaptureStatus(s, 10)
                                }
                    }
            }, e.prototype.storeCommandIds = function () {
                for (var e = ["unusedCommandIds", "disabledCommandIds", "redundantCommandIds", "validCommandIds"], t = 0, n = e; t < n.length; t++) {
                    var r = n[t];
                    this.currentState[r] = []
                }
                for (var a in this.capturedCommandsByState)
                    if (this.capturedCommandsByState.hasOwnProperty(a))
                        for (var o = 0, i = this.capturedCommandsByState[a]; o < i.length; o++) {
                            var s = i[o];
                            switch (s.status) {
                                case 10:
                                    this.currentState.unusedCommandIds.push(s.id);
                                    break;
                                case 20:
                                    this.currentState.disabledCommandIds.push(s.id);
                                    break;
                                case 30:
                                    this.currentState.redundantCommandIds.push(s.id);
                                    break;
                                case 40:
                                    this.currentState.validCommandIds.push(s.id)
                            }
                        }
                for (var u = 0, c = e; u < c.length; u++) r = c[u], this.currentState[r].length || delete this.currentState[r]
            }, e.prototype.changeCommandCaptureStatus = function (e, t) {
                return e.status < t && (e.status = t, !0)
            }, e.prototype.areStatesEquals = function (e, t) {
                if (typeof e != typeof t) return !1;
                if (e && !t) return !1;
                if (t && !e) return !1;
                if (null == e) return !0;
                if (e.length && t.length && "string" != typeof e) {
                    if (e.length !== t.length) return !1;
                    for (var n = 0; n < e.length; n++)
                        if (e[n] !== t[n]) return !1;
                    return !0
                }
                return e === t
            }, e.prototype.isStateEnable = function (e, t) {
                return !0
            }, e.prototype.getSpectorData = function (e) {
                if (e) return {
                    __SPECTOR_Object_TAG: N.getWebGlObjectTag(e) || this.options.tagWebGlObject(e),
                    __SPECTOR_Object_CustomData: e.__SPECTOR_Object_CustomData,
                    __SPECTOR_Metadata: e.__SPECTOR_Metadata
                }
            }, e.prototype.readFromContextNoSideEffects = function () {
                this.toggleCapture(!1), this.readFromContext(), this.toggleCapture(!0)
            }, e.prototype.isStateEnableNoSideEffects = function (e, t) {
                this.toggleCapture(!1);
                var n = this.isStateEnable(e, t);
                return this.toggleCapture(!0), n
            }, e.prototype.getCommandNameToStates = function () {
                var e = {};
                for (var t in this.changeCommandsByState)
                    if (this.changeCommandsByState.hasOwnProperty(t))
                        for (var n = 0, r = this.changeCommandsByState[t]; n < r.length; n++) {
                            var a = r[n];
                            e[a] = e[a] || [], e[a].push(t)
                        }
                return e
            }, e
        }(),
        Le = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        be = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Le(t, e), t.prototype.getWebgl1Parameters = function () {
                return []
            }, t.prototype.getWebgl2Parameters = function () {
                return []
            }, t.prototype.getChangeCommandsByState = function () {
                this.parameters = [], this.parameters.push(this.getWebgl1Parameters()), this.contextVersion > 1 && this.parameters.push(this.getWebgl2Parameters());
                for (var e = {}, t = 1; t <= this.contextVersion && !(t > this.parameters.length); t++)
                    if (this.parameters[t - 1])
                        for (var n = 0, r = this.parameters[t - 1]; n < r.length; n++) {
                            var a = r[n];
                            if (a.changeCommands)
                                for (var o = 0, i = a.changeCommands; o < i.length; o++) {
                                    var s = i[o];
                                    e[a.constant.name] = e[a.constant.name] || [], e[a.constant.name].push(s)
                                }
                        }
                return e
            }, t.prototype.readFromContext = function () {
                for (var e = 1; e <= this.contextVersion && !(e > this.parameters.length); e++)
                    for (var t = 0, n = this.parameters[e - 1]; t < n.length; t++) {
                        var r = n[t],
                            a = this.readParameterFromContext(r),
                            o = N.getWebGlObjectTag(a);
                        if (o) this.currentState[r.constant.name] = o;
                        else {
                            var i = this.stringifyParameterValue(a, r);
                            this.currentState[r.constant.name] = i
                        }
                    }
            }, t.prototype.readParameterFromContext = function (e) {
                return e.constant.extensionName && !this.extensions[e.constant.extensionName] ? "Extension ".concat(e.constant.extensionName, " is unavailable.") : this.context.getParameter(e.constant.value)
            }, t.prototype.stringifyParameterValue = function (e, t) {
                if (null === e) return "null";
                if (void 0 === e) return "undefined";
                if (30 === t.returnType) return me(e);
                if ("number" == typeof e && m.isWebGlConstant(e)) {
                    if (20 === t.returnType) {
                        var n = t.changeCommands && t.changeCommands[0] || "";
                        return m.stringifyWebGlConstant(e, n)
                    }
                    return e
                }
                if (e.length && "string" != typeof e) {
                    for (var r = [], a = 0; a < e.length; a++) r.push(e[a]);
                    return r
                }
                return e
            }, t
        }(Me),
        Ue = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        De = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Ue(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.PACK_ALIGNMENT,
                    changeCommands: ["pixelStorei"]
                }, {
                    constant: m.UNPACK_ALIGNMENT,
                    changeCommands: ["pixelStorei"]
                }, {
                    constant: m.UNPACK_COLORSPACE_CONVERSION_WEBGL,
                    returnType: 20,
                    changeCommands: ["pixelStorei"]
                }, {
                    constant: m.UNPACK_FLIP_Y_WEBGL,
                    changeCommands: ["pixelStorei"]
                }, {
                    constant: m.UNPACK_PREMULTIPLY_ALPHA_WEBGL,
                    changeCommands: ["pixelStorei"]
                }]
            }, t.prototype.getWebgl2Parameters = function () {
                return [{
                    constant: m.PACK_ROW_LENGTH,
                    changeCommands: ["pixelStorei"]
                }, {
                    constant: m.PACK_SKIP_PIXELS,
                    changeCommands: ["pixelStorei"]
                }, {
                    constant: m.PACK_SKIP_ROWS,
                    changeCommands: ["pixelStorei"]
                }, {
                    constant: m.UNPACK_IMAGE_HEIGHT,
                    changeCommands: ["pixelStorei"]
                }, {
                    constant: m.UNPACK_SKIP_PIXELS,
                    changeCommands: ["pixelStorei"]
                }, {
                    constant: m.UNPACK_SKIP_ROWS,
                    changeCommands: ["pixelStorei"]
                }, {
                    constant: m.UNPACK_SKIP_IMAGES,
                    changeCommands: ["pixelStorei"]
                }]
            }, t.prototype.getConsumeCommands = function () {
                return ["readPixels", "texImage2D", "texSubImage2D"]
            }, t.prototype.isValidChangeCommand = function (e, t) {
                return T[t].value === e.commandArguments[0]
            }, t.stateName = "AlignmentState", t
        }(be),
        Ge = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        xe = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Ge(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.BLEND,
                    changeCommands: ["enable", "disable"]
                }, {
                    constant: m.BLEND_COLOR,
                    changeCommands: ["blendColor"]
                }, {
                    constant: m.BLEND_DST_ALPHA,
                    returnType: 20,
                    changeCommands: ["blendFunc", "blendFuncSeparate"]
                }, {
                    constant: m.BLEND_DST_RGB,
                    returnType: 20,
                    changeCommands: ["blendFunc", "blendFuncSeparate"]
                }, {
                    constant: m.BLEND_EQUATION_ALPHA,
                    returnType: 20,
                    changeCommands: ["blendEquation", "blendEquationSeparate"]
                }, {
                    constant: m.BLEND_EQUATION_RGB,
                    returnType: 20,
                    changeCommands: ["blendEquation", "blendEquationSeparate"]
                }, {
                    constant: m.BLEND_SRC_ALPHA,
                    returnType: 20,
                    changeCommands: ["blendFunc", "blendFuncSeparate"]
                }, {
                    constant: m.BLEND_SRC_RGB,
                    returnType: 20,
                    changeCommands: ["blendFunc", "blendFuncSeparate"]
                }]
            }, t.prototype.isValidChangeCommand = function (e, t) {
                return "enable" !== e.name && "disable" !== e.name || e.commandArguments[0] === m.BLEND.value
            }, t.prototype.getConsumeCommands = function () {
                return _
            }, t.prototype.isStateEnable = function (e, t) {
                return this.context.isEnabled(m.BLEND.value)
            }, t.stateName = "BlendState", t
        }(be),
        we = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Xe = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return we(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.COLOR_CLEAR_VALUE,
                    changeCommands: ["clearColor"]
                }, {
                    constant: m.DEPTH_CLEAR_VALUE,
                    changeCommands: ["clearDepth"]
                }, {
                    constant: m.STENCIL_CLEAR_VALUE,
                    changeCommands: ["clearStencil"]
                }]
            }, t.prototype.getConsumeCommands = function () {
                return ["clear"]
            }, t.prototype.isStateEnable = function (e, t) {
                switch (e) {
                    case m.COLOR_CLEAR_VALUE.name:
                        return m.COLOR_BUFFER_BIT.value === (t[0] & m.COLOR_BUFFER_BIT.value);
                    case m.DEPTH_CLEAR_VALUE.name:
                        return m.DEPTH_BUFFER_BIT.value === (t[0] & m.DEPTH_BUFFER_BIT.value);
                    case m.STENCIL_CLEAR_VALUE.name:
                        return m.STENCIL_BUFFER_BIT.value === (t[0] & m.STENCIL_BUFFER_BIT.value)
                }
                return !1
            }, t.stateName = "ClearState", t
        }(be),
        We = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Ve = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return We(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.COLOR_WRITEMASK,
                    changeCommands: ["colorMask"]
                }]
            }, t.prototype.getConsumeCommands = function () {
                return _
            }, t.stateName = "ColorState", t
        }(be),
        He = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        je = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return He(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.SAMPLE_COVERAGE_VALUE,
                    changeCommands: ["sampleCoverage"]
                }, {
                    constant: m.SAMPLE_COVERAGE_INVERT,
                    changeCommands: ["sampleCoverage"]
                }]
            }, t.prototype.getWebgl2Parameters = function () {
                return [{
                    constant: m.SAMPLE_COVERAGE,
                    changeCommands: ["enable", "disable"]
                }, {
                    constant: m.SAMPLE_ALPHA_TO_COVERAGE,
                    changeCommands: ["enable", "disable"]
                }]
            }, t.prototype.isValidChangeCommand = function (e, t) {
                return "enable" !== e.name && "disable" !== e.name || (e.commandArguments[0] === m.SAMPLE_COVERAGE.value ? t === m.SAMPLE_COVERAGE.name : e.commandArguments[0] === m.SAMPLE_ALPHA_TO_COVERAGE.value && t === m.SAMPLE_ALPHA_TO_COVERAGE.name)
            }, t.prototype.getConsumeCommands = function () {
                return _
            }, t.prototype.isStateEnable = function (e, t) {
                return 2 === this.contextVersion && this.context.isEnabled(m.SAMPLE_COVERAGE.value)
            }, t.stateName = "CoverageState", t
        }(be),
        Ye = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Ke = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Ye(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.CULL_FACE,
                    changeCommands: ["enable", "disable"]
                }, {
                    constant: m.CULL_FACE_MODE,
                    returnType: 20,
                    changeCommands: ["cullFace"]
                }]
            }, t.prototype.getConsumeCommands = function () {
                return _
            }, t.prototype.isValidChangeCommand = function (e, t) {
                return "enable" !== e.name && "disable" !== e.name || e.commandArguments[0] === m.CULL_FACE.value
            }, t.prototype.isStateEnable = function (e, t) {
                return this.context.isEnabled(m.CULL_FACE.value)
            }, t.stateName = "CullState", t
        }(be),
        ke = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Ze = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return ke(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.DEPTH_TEST,
                    changeCommands: ["enable", "disable"]
                }, {
                    constant: m.DEPTH_FUNC,
                    returnType: 20,
                    changeCommands: ["depthFunc"]
                }, {
                    constant: m.DEPTH_RANGE,
                    changeCommands: ["depthRange"]
                }, {
                    constant: m.DEPTH_WRITEMASK,
                    changeCommands: ["depthMask"]
                }]
            }, t.prototype.getConsumeCommands = function () {
                return _
            }, t.prototype.isValidChangeCommand = function (e, t) {
                return "enable" !== e.name && "disable" !== e.name || e.commandArguments[0] === m.DEPTH_TEST.value
            }, t.prototype.isStateEnable = function (e, t) {
                return this.context.isEnabled(m.DEPTH_TEST.value)
            }, t.stateName = "DepthState", t
        }(be),
        ze = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        qe = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return ze(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.DITHER,
                    changeCommands: ["enable", "disable"]
                }, {
                    constant: m.VIEWPORT,
                    changeCommands: ["viewPort"]
                }, {
                    constant: m.FRONT_FACE,
                    returnType: 20,
                    changeCommands: ["frontFace"]
                }, {
                    constant: m.FRAGMENT_SHADER_DERIVATIVE_HINT_OES,
                    changeCommands: ["hint"]
                }]
            }, t.prototype.getWebgl2Parameters = function () {
                return [{
                    constant: m.RASTERIZER_DISCARD,
                    changeCommands: ["enable", "disable"]
                }, {
                    constant: m.FRAGMENT_SHADER_DERIVATIVE_HINT,
                    changeCommands: ["hint"]
                }]
            }, t.prototype.isValidChangeCommand = function (e, t) {
                return "enable" === e.name || "disable" === e.name ? e.commandArguments[0] === m.DITHER.value ? t === m.DITHER.name : e.commandArguments[0] === m.RASTERIZER_DISCARD.value && t === m.RASTERIZER_DISCARD.name : "hint" !== e.name || (e.commandArguments[0] === m.FRAGMENT_SHADER_DERIVATIVE_HINT_OES.value ? t === m.FRAGMENT_SHADER_DERIVATIVE_HINT_OES.name : e.commandArguments[0] === m.FRAGMENT_SHADER_DERIVATIVE_HINT.value && t === m.FRAGMENT_SHADER_DERIVATIVE_HINT.name)
            }, t.prototype.getConsumeCommands = function () {
                return _
            }, t.prototype.isStateEnable = function (e, t) {
                switch (e) {
                    case m.DITHER.name:
                        return this.context.isEnabled(m.DITHER.value);
                    case m.RASTERIZER_DISCARD.name:
                        return this.context.isEnabled(m.RASTERIZER_DISCARD.value)
                }
                return !0
            }, t.stateName = "DrawState", t
        }(be),
        Qe = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Je = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Qe(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.GENERATE_MIPMAP_HINT,
                    changeCommands: ["hint"]
                }]
            }, t.prototype.getConsumeCommands = function () {
                return ["generateMipmap"]
            }, t.stateName = "MipmapHintState", t
        }(be),
        $e = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        et = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return $e(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.POLYGON_OFFSET_FILL,
                    changeCommands: ["enable", "disable"]
                }, {
                    constant: m.POLYGON_OFFSET_FACTOR,
                    changeCommands: ["polygonOffset"]
                }, {
                    constant: m.POLYGON_OFFSET_UNITS,
                    changeCommands: ["polygonOffset"]
                }]
            }, t.prototype.isValidChangeCommand = function (e, t) {
                return "enable" !== e.name && "disable" !== e.name || e.commandArguments[0] === m.POLYGON_OFFSET_FILL.value
            }, t.prototype.getConsumeCommands = function () {
                return _
            }, t.prototype.isStateEnable = function (e, t) {
                return this.context.isEnabled(m.POLYGON_OFFSET_FILL.value)
            }, t.stateName = "PolygonOffsetState", t
        }(be),
        tt = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        nt = function (e, t, n) {
            if (n || 2 === arguments.length)
                for (var r, a = 0, o = t.length; a < o; a++) !r && a in t || (r || (r = Array.prototype.slice.call(t, 0, a)), r[a] = t[a]);
            return e.concat(r || Array.prototype.slice.call(t))
        },
        rt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return tt(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.SCISSOR_TEST,
                    changeCommands: ["enable", "disable"]
                }, {
                    constant: m.SCISSOR_BOX,
                    changeCommands: ["scissor"]
                }]
            }, t.prototype.isValidChangeCommand = function (e, t) {
                return "enable" !== e.name && "disable" !== e.name || e.commandArguments[0] === m.SCISSOR_TEST.value
            }, t.prototype.getConsumeCommands = function () {
                return nt(nt([], _, !0), ["clear"], !1)
            }, t.prototype.isStateEnable = function (e, t) {
                return this.context.isEnabled(m.SCISSOR_TEST.value)
            }, t.stateName = "ScissorState", t
        }(be),
        at = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        ot = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return at(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.STENCIL_TEST,
                    changeCommands: ["enable", "disable"]
                }, {
                    constant: m.STENCIL_BACK_FAIL,
                    returnType: 20,
                    changeCommands: ["stencilOp", "stencilOpSeparate"]
                }, {
                    constant: m.STENCIL_BACK_FUNC,
                    returnType: 20,
                    changeCommands: ["stencilFunc", "stencilFuncSeparate"]
                }, {
                    constant: m.STENCIL_BACK_PASS_DEPTH_FAIL,
                    returnType: 20,
                    changeCommands: ["stencilOp", "stencilOpSeparate"]
                }, {
                    constant: m.STENCIL_BACK_PASS_DEPTH_PASS,
                    returnType: 20,
                    changeCommands: ["stencilOp", "stencilOpSeparate"]
                }, {
                    constant: m.STENCIL_BACK_REF,
                    changeCommands: ["stencilFunc", "stencilFuncSeparate"]
                }, {
                    constant: m.STENCIL_BACK_VALUE_MASK,
                    returnType: 30,
                    changeCommands: ["stencilFunc", "stencilFuncSeparate"]
                }, {
                    constant: m.STENCIL_BACK_WRITEMASK,
                    returnType: 30,
                    changeCommands: ["stencilMask", "stencilMaskSeparate"]
                }, {
                    constant: m.STENCIL_FAIL,
                    returnType: 20,
                    changeCommands: ["stencilOp", "stencilOpSeparate"]
                }, {
                    constant: m.STENCIL_FUNC,
                    returnType: 20,
                    changeCommands: ["stencilFunc", "stencilFuncSeparate"]
                }, {
                    constant: m.STENCIL_PASS_DEPTH_FAIL,
                    returnType: 20,
                    changeCommands: ["stencilOp", "stencilOpSeparate"]
                }, {
                    constant: m.STENCIL_PASS_DEPTH_PASS,
                    returnType: 20,
                    changeCommands: ["stencilOp", "stencilOpSeparate"]
                }, {
                    constant: m.STENCIL_REF,
                    changeCommands: ["stencilFunc", "stencilFuncSeparate"]
                }, {
                    constant: m.STENCIL_VALUE_MASK,
                    returnType: 30,
                    changeCommands: ["stencilFunc", "stencilFuncSeparate"]
                }, {
                    constant: m.STENCIL_WRITEMASK,
                    returnType: 30,
                    changeCommands: ["stencilMask", "stencilMaskSeparate"]
                }]
            }, t.prototype.readFromContext = function () {
                e.prototype.readFromContext.call(this);
                var t = this.context,
                    n = m.FRAMEBUFFER.value,
                    r = m.STENCIL_ATTACHMENT.value,
                    a = 0;
                t.getParameter(m.FRAMEBUFFER_BINDING.value) ? this.context.getFramebufferAttachmentParameter(n, r, m.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE.value) !== m.NONE.value && (this.contextVersion > 1 ? a = this.context.getFramebufferAttachmentParameter(n, r, m.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE.value) : this.context.getFramebufferAttachmentParameter(n, r, m.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME.value) === m.RENDERBUFFER.value && (a = t.getRenderbufferParameter(t.RENDERBUFFER, t.RENDERBUFFER_STENCIL_SIZE))) : a = this.readParameterFromContext({
                    constant: m.STENCIL_BITS
                }), this.currentState[m.STENCIL_BITS.name] = "" + a
            }, t.prototype.isValidChangeCommand = function (e, n) {
                return "enable" === e.name || "disable" === e.name ? e.commandArguments[0] === m.STENCIL_TEST.value : "stencilOp" === e.name || "stencilOpSeparate" === e.name ? t.stencilOpStates.indexOf(e.commandArguments[0]) > 0 : "stencilFunc" === e.name || "stencilFuncSeparate" === e.name ? t.stencilFuncStates.indexOf(e.commandArguments[0]) > 0 : "stencilMask" !== e.name && "stencilMaskSeparate" !== e.name || t.stencilMaskStates.indexOf(e.commandArguments[0]) > 0
            }, t.prototype.getConsumeCommands = function () {
                return _
            }, t.prototype.isStateEnable = function (e, t) {
                return this.context.isEnabled(m.STENCIL_TEST.value)
            }, t.stateName = "StencilState", t.stencilOpStates = [m.STENCIL_BACK_FAIL.value, m.STENCIL_BACK_PASS_DEPTH_FAIL.value, m.STENCIL_BACK_PASS_DEPTH_PASS.value, m.STENCIL_FAIL.value, m.STENCIL_PASS_DEPTH_FAIL.value, m.STENCIL_PASS_DEPTH_PASS.value], t.stencilFuncStates = [m.STENCIL_BACK_FUNC.value, m.STENCIL_BACK_REF.value, m.STENCIL_BACK_VALUE_MASK.value, m.STENCIL_FUNC.value, m.STENCIL_REF.value, m.STENCIL_VALUE_MASK.value], t.stencilMaskStates = [m.STENCIL_BACK_WRITEMASK.value, m.STENCIL_WRITEMASK.value], t
        }(be),
        it = function () {
            function e() {}
            return e.isSupportedCombination = function (e, t, n) {
                return e = e || m.UNSIGNED_BYTE.value, ((t = t || m.RGBA.value) === m.RGB.value || t === m.RGBA.value || t === m.RED.value) && (n === m.RGB.value || n === m.RGBA.value || n === m.RGBA8.value || n === m.RGBA16F.value || n === m.RGBA32F.value || n === m.RGB16F.value || n === m.RGB32F.value || n === m.R11F_G11F_B10F.value || n === m.SRGB8.value || n === m.SRGB8_ALPHA8.value || n === m.R8.value) && this.isSupportedComponentType(e)
            }, e.readPixels = function (e, t, n, r, a, o, i) {
                var s;
                if (e.getError(), i == m.RED.value && e.RED) {
                    var u = r * a,
                        c = void 0;
                    o === m.UNSIGNED_BYTE.value ? (c = new Uint8Array(u), s = new Uint8Array(4 * u)) : (o = m.FLOAT.value, c = new Float32Array(u), s = new Uint8Array(4 * u)), e.readPixels(t, n, r, a, e.RED, o, c);
                    for (var E = 0; E < c.length; E++) s[4 * E] = c[E], o === m.UNSIGNED_BYTE.value ? (s[4 * E + 1] = 0, s[4 * E + 2] = 0, s[4 * E + 3] = 255) : (s[4 * E + 1] = 0, s[4 * E + 2] = 0, s[4 * E + 3] = 1)
                } else u = r * a * 4, o === m.UNSIGNED_BYTE.value ? s = new Uint8Array(u) : (o = m.FLOAT.value, s = new Float32Array(u)), e.readPixels(t, n, r, a, e.RGBA, o, s);
                if (!e.getError()) {
                    if (o === m.UNSIGNED_BYTE.value) return s;
                    var _ = new Uint8Array(r * a * 4);
                    for (E = 0; E < a; E++)
                        for (var p = 0; p < r; p++) _[E * r * 4 + 4 * p + 0] = 255 * Math.min(Math.max(s[E * r * 4 + 4 * p + 0], 0), 1), _[E * r * 4 + 4 * p + 1] = 255 * Math.min(Math.max(s[E * r * 4 + 4 * p + 1], 0), 1), _[E * r * 4 + 4 * p + 2] = 255 * Math.min(Math.max(s[E * r * 4 + 4 * p + 2], 0), 1), _[E * r * 4 + 4 * p + 3] = 255 * Math.min(Math.max(s[E * r * 4 + 4 * p + 3], 0), 1);
                    return _
                }
            }, e.isSupportedComponentType = function (e) {
                return e === m.UNSIGNED_BYTE.value || e === m.UNSIGNED_SHORT_4_4_4_4.value || e === m.UNSIGNED_SHORT_5_5_5_1.value || e === m.UNSIGNED_SHORT_5_6_5.value || e === m.HALF_FLOAT.value || e === m.HALF_FLOAT_OES.value || e === m.FLOAT.value
            }, e
        }(),
        st = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        ut = function (e) {
            function t(t) {
                var n = e.call(this, t) || this;
                return n.captureFrameBuffer = t.context.createFramebuffer(), n.workingCanvas = document.createElement("canvas"), n.workingContext2D = n.workingCanvas.getContext("2d"), n.captureCanvas = document.createElement("canvas"), n.captureContext2D = n.captureCanvas.getContext("2d"), n.captureContext2D.imageSmoothingEnabled = !0, n.captureContext2D.mozImageSmoothingEnabled = !0, n.captureContext2D.oImageSmoothingEnabled = !0, n.captureContext2D.webkitImageSmoothingEnabled = !0, n.captureContext2D.msImageSmoothingEnabled = !0, n
            }
            return st(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getConsumeCommands = function () {
                return function (e, t, n) {
                    if (n || 2 === arguments.length)
                        for (var r, a = 0, o = t.length; a < o; a++) !r && a in t || (r || (r = Array.prototype.slice.call(t, 0, a)), r[a] = t[a]);
                    return e.concat(r || Array.prototype.slice.call(t))
                }(["clear", "clearBufferfv", "clearBufferiv", "clearBufferuiv", "clearBufferfi"], _, !0)
            }, t.prototype.readFromContext = function () {
                var e = this.context;
                this.currentState.Attachments = [];
                var t = this.context.getParameter(m.FRAMEBUFFER_BINDING.value);
                if (!t) return this.currentState.FrameBuffer = null, void this.getCapture(e, "Canvas COLOR_ATTACHMENT", 0, 0, e.drawingBufferWidth, e.drawingBufferHeight, 0, 0, m.UNSIGNED_BYTE.value);
                var n = e.getParameter(e.VIEWPORT),
                    r = n[0],
                    a = n[1],
                    o = n[2],
                    i = n[3],
                    s = this.context.getFramebufferAttachmentParameter(e.FRAMEBUFFER, m.COLOR_ATTACHMENT0.value, m.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME.value);
                this.currentState.FrameBuffer = this.getSpectorData(s) || this.getSpectorData(t), t.AA = ct++, t.__SPECTOR_Object_TAG && (t.__SPECTOR_Object_TAG.AA = ct++);
                var u = this.context.checkFramebufferStatus(m.FRAMEBUFFER.value);
                if (this.currentState.FrameBufferStatus = R[u].name, u === m.FRAMEBUFFER_COMPLETE.value)
                    if (this.extensions[m.MAX_DRAW_BUFFERS_WEBGL.extensionName])
                        for (var c = this.context.getParameter(m.MAX_DRAW_BUFFERS_WEBGL.value), E = 0; E < c; E++) this.readFrameBufferAttachmentFromContext(this.context, t, T["COLOR_ATTACHMENT" + E + "_WEBGL"], r, a, o, i);
                    else if (this.contextVersion > 1)
                    for (c = this.context.getParameter(m.MAX_DRAW_BUFFERS.value), E = 0; E < c; E++) this.readFrameBufferAttachmentFromContext(this.context, t, T["COLOR_ATTACHMENT" + E], r, a, o, i);
                else this.readFrameBufferAttachmentFromContext(this.context, t, T.COLOR_ATTACHMENT0, r, a, o, i)
            }, t.prototype.readFrameBufferAttachmentFromContext = function (e, t, n, r, a, o, i) {
                var s = m.FRAMEBUFFER.value,
                    u = this.context.getFramebufferAttachmentParameter(s, n.value, m.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE.value);
                if (u !== m.NONE.value) {
                    var c = this.context.getFramebufferAttachmentParameter(s, n.value, m.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME.value);
                    if (c) {
                        var E = this.contextVersion > 1 ? this.context.getFramebufferAttachmentParameter(s, n.value, m.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE.value) : m.UNSIGNED_BYTE.value;
                        u === m.RENDERBUFFER.value ? this.readFrameBufferAttachmentFromRenderBuffer(e, t, n, r, a, o, i, s, E, c) : u === m.TEXTURE.value && this.readFrameBufferAttachmentFromTexture(e, t, n, r, a, o, i, s, E, c)
                    }
                }
            }, t.prototype.readFrameBufferAttachmentFromRenderBuffer = function (e, t, n, r, a, o, i, s, u, c) {
                var E = 0,
                    _ = 0;
                if (c.__SPECTOR_Object_CustomData) {
                    var p = c.__SPECTOR_Object_CustomData;
                    if (o = p.width, i = p.height, E = p.samples, _ = p.internalFormat, !E && !it.isSupportedCombination(u, m.RGBA.value, _)) return
                } else o += r, i += a;
                if (r = a = 0, E) {
                    var l = e,
                        T = e.createRenderbuffer(),
                        R = e.getParameter(e.RENDERBUFFER_BINDING);
                    e.bindRenderbuffer(e.RENDERBUFFER, T), e.renderbufferStorage(e.RENDERBUFFER, _, o, i), e.bindRenderbuffer(e.RENDERBUFFER, R), e.bindFramebuffer(m.FRAMEBUFFER.value, this.captureFrameBuffer), e.framebufferRenderbuffer(m.FRAMEBUFFER.value, m.COLOR_ATTACHMENT0.value, m.RENDERBUFFER.value, T);
                    var f = l.getParameter(l.READ_FRAMEBUFFER_BINDING),
                        A = l.getParameter(l.DRAW_FRAMEBUFFER_BINDING);
                    l.bindFramebuffer(l.READ_FRAMEBUFFER, t), l.bindFramebuffer(l.DRAW_FRAMEBUFFER, this.captureFrameBuffer), l.blitFramebuffer(0, 0, o, i, 0, 0, o, i, e.COLOR_BUFFER_BIT, e.NEAREST), l.bindFramebuffer(m.FRAMEBUFFER.value, this.captureFrameBuffer), l.bindFramebuffer(l.READ_FRAMEBUFFER, f), l.bindFramebuffer(l.DRAW_FRAMEBUFFER, A), this.context.checkFramebufferStatus(m.FRAMEBUFFER.value) === m.FRAMEBUFFER_COMPLETE.value && this.getCapture(e, n.name, r, a, o, i, 0, 0, m.UNSIGNED_BYTE.value), e.bindFramebuffer(m.FRAMEBUFFER.value, t), e.deleteRenderbuffer(T)
                } else e.bindFramebuffer(m.FRAMEBUFFER.value, this.captureFrameBuffer), e.framebufferRenderbuffer(m.FRAMEBUFFER.value, m.COLOR_ATTACHMENT0.value, m.RENDERBUFFER.value, c), this.context.checkFramebufferStatus(m.FRAMEBUFFER.value) === m.FRAMEBUFFER_COMPLETE.value && this.getCapture(e, n.name, r, a, o, i, 0, 0, u), e.bindFramebuffer(m.FRAMEBUFFER.value, t)
            }, t.prototype.readFrameBufferAttachmentFromTexture = function (e, t, n, r, a, o, i, s, u, c) {
                var E = 0;
                this.contextVersion > 1 && (E = this.context.getFramebufferAttachmentParameter(s, n.value, m.FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER.value));
                var _ = this.context.getFramebufferAttachmentParameter(s, n.value, m.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL.value),
                    p = this.context.getFramebufferAttachmentParameter(s, n.value, m.FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE.value),
                    l = (p > 0 ? R[p].name : m.TEXTURE_2D.name, !1),
                    T = u;
                if (c.__SPECTOR_Object_CustomData) {
                    var f = c.__SPECTOR_Object_CustomData;
                    if (o = f.width, i = f.height, T = f.type || m.UNSIGNED_BYTE.value, l = f.target === m.TEXTURE_2D_ARRAY.name, !it.isSupportedCombination(f.type, f.format, f.internalFormat)) return
                } else o += r, i += a;
                r = a = 0, e.bindFramebuffer(m.FRAMEBUFFER.value, this.captureFrameBuffer), E > 0 || l ? e.framebufferTextureLayer(m.FRAMEBUFFER.value, m.COLOR_ATTACHMENT0.value, c, _, E) : e.framebufferTexture2D(m.FRAMEBUFFER.value, m.COLOR_ATTACHMENT0.value, p || m.TEXTURE_2D.value, c, _), this.context.checkFramebufferStatus(m.FRAMEBUFFER.value) === m.FRAMEBUFFER_COMPLETE.value && this.getCapture(e, n.name, r, a, o, i, p, E, T), e.bindFramebuffer(m.FRAMEBUFFER.value, t)
            }, t.prototype.getCapture = function (e, n, r, a, i, s, u, c, E) {
                var _ = {
                    attachmentName: n,
                    src: null,
                    textureCubeMapFace: u ? R[u].name : null,
                    textureLayer: c
                };
                if (!this.quickCapture) try {
                    var p = it.readPixels(e, r, a, i, s, E, 0);
                    if (p) {
                        this.workingCanvas.width = i, this.workingCanvas.height = s;
                        var l = this.workingContext2D.createImageData(Math.ceil(i), Math.ceil(s));
                        if (l.data.set(p), this.workingContext2D.putImageData(l, 0, 0), this.fullCapture) this.captureCanvas.width = this.workingCanvas.width, this.captureCanvas.height = this.workingCanvas.height;
                        else {
                            var m = i / s;
                            m < 1 ? (this.captureCanvas.width = t.captureBaseSize * m, this.captureCanvas.height = t.captureBaseSize) : m > 1 ? (this.captureCanvas.width = t.captureBaseSize, this.captureCanvas.height = t.captureBaseSize / m) : (this.captureCanvas.width = t.captureBaseSize, this.captureCanvas.height = t.captureBaseSize)
                        }
                        this.captureCanvas.width = Math.max(this.captureCanvas.width, 1), this.captureCanvas.height = Math.max(this.captureCanvas.height, 1), this.captureContext2D.globalCompositeOperation = "copy", this.captureContext2D.scale(1, -1), this.captureContext2D.translate(0, -this.captureCanvas.height), this.captureContext2D.drawImage(this.workingCanvas, 0, 0, i, s, 0, 0, this.captureCanvas.width, this.captureCanvas.height), this.captureContext2D.setTransform(1, 0, 0, 1, 0, 0), this.captureContext2D.globalCompositeOperation = "source-over", _.src = this.captureCanvas.toDataURL()
                    }
                } catch (e) {
                    o.warn("Spector can not capture the visual state: " + e)
                }
                this.currentState.Attachments.push(_)
            }, t.prototype.analyse = function (e) {}, t.stateName = "VisualState", t.captureBaseSize = 256, t
        }(Me),
        ct = 0,
        Et = function () {
            function e(e) {
                this.context = e.context, this.captureFrameBuffer = e.context.createFramebuffer(), this.workingCanvas = document.createElement("canvas"), this.workingContext2D = this.workingCanvas.getContext("2d"), this.captureCanvas = document.createElement("canvas"), this.captureContext2D = this.captureCanvas.getContext("2d"), this._setSmoothing(!0)
            }
            return e.prototype.appendTextureState = function (e, t, n, r) {
                if (void 0 === n && (n = null), t) {
                    var a = t.__SPECTOR_Object_CustomData;
                    if (a && (this.fullCapture = r, a.type && (e.textureType = this.getWebGlConstant(a.type)), a.format && (e.format = this.getWebGlConstant(a.format)), a.internalFormat && (e.internalFormat = this.getWebGlConstant(a.internalFormat)), e.width = a.width, e.height = a.height, a.depth && (e.depth = a.depth), n)) {
                        var o = "NEAREST" === e.samplerMagFilter || "NEAREST" === e.magFilter;
                        e.visual = this.getTextureVisualState(n, t, a, o)
                    }
                }
            }, e.prototype.getTextureVisualState = function (t, n, r, a) {
                try {
                    var o = this.context,
                        i = {};
                    if (!it.isSupportedCombination(r.type, r.format, r.internalFormat)) return i;
                    var s = this.context.getParameter(m.FRAMEBUFFER_BINDING.value);
                    o.bindFramebuffer(m.FRAMEBUFFER.value, this.captureFrameBuffer);
                    try {
                        var u = r.width,
                            c = r.height;
                        if (t === m.TEXTURE_3D && r.depth)
                            for (var E = o, _ = 0; _ < r.depth; _++) _ > 2 && _ < r.depth - 3 || (E.framebufferTextureLayer(m.FRAMEBUFFER.value, m.COLOR_ATTACHMENT0.value, n, 0, _), i["3D Layer " + _] = this.getCapture(o, 0, 0, u, c, r.type, a, r.format));
                        else if (t === m.TEXTURE_2D_ARRAY && r.depth)
                            for (E = o, _ = 0; _ < r.depth; _++) _ > 2 && _ < r.depth - 3 || (E.framebufferTextureLayer(m.FRAMEBUFFER.value, m.COLOR_ATTACHMENT0.value, n, 0, _), i["Layer " + _] = this.getCapture(o, 0, 0, u, c, r.type, a, r.format));
                        else if (t === m.TEXTURE_CUBE_MAP)
                            for (var p = 0, l = e.cubeMapFaces; p < l.length; p++) {
                                var T = l[p];
                                o.framebufferTexture2D(m.FRAMEBUFFER.value, m.COLOR_ATTACHMENT0.value, T.value, n, 0), i[T.name] = this.getCapture(o, 0, 0, u, c, r.type, a, r.format)
                            } else o.framebufferTexture2D(m.FRAMEBUFFER.value, m.COLOR_ATTACHMENT0.value, m.TEXTURE_2D.value, n, 0), i[m.TEXTURE_2D.name] = this.getCapture(o, 0, 0, u, c, r.type, a, r.format)
                    } catch (e) {}
                    return o.bindFramebuffer(m.FRAMEBUFFER.value, s), i
                } catch (e) {}
            }, e.prototype.getCapture = function (e, t, n, r, a, o, i, s) {
                try {
                    if (this.context.checkFramebufferStatus(m.FRAMEBUFFER.value) !== m.FRAMEBUFFER_COMPLETE.value) return;
                    o = o || m.UNSIGNED_BYTE.value;
                    var u = it.readPixels(e, t, n, r, a, o, s);
                    if (!u) return;
                    this.workingCanvas.width = r, this.workingCanvas.height = a;
                    var c = this.workingContext2D.createImageData(r, a);
                    if (c.data.set(u), this.workingContext2D.putImageData(c, 0, 0), this.fullCapture) this.captureCanvas.width = this.workingCanvas.width, this.captureCanvas.height = this.workingCanvas.height;
                    else {
                        var E = r / a;
                        E < 1 ? (this.captureCanvas.width = ut.captureBaseSize * E, this.captureCanvas.height = ut.captureBaseSize) : E > 1 ? (this.captureCanvas.width = ut.captureBaseSize, this.captureCanvas.height = ut.captureBaseSize / E) : (this.captureCanvas.width = ut.captureBaseSize, this.captureCanvas.height = ut.captureBaseSize)
                    }
                    return this.captureCanvas.width = Math.max(this.captureCanvas.width, 1), this.captureCanvas.height = Math.max(this.captureCanvas.height, 1), this.captureContext2D.globalCompositeOperation = "copy", this.captureContext2D.scale(1, -1), this.captureContext2D.translate(0, -this.captureCanvas.height), this._setSmoothing(!i), this.captureContext2D.drawImage(this.workingCanvas, 0, 0, r, a, 0, 0, this.captureCanvas.width, this.captureCanvas.height), this.captureContext2D.setTransform(1, 0, 0, 1, 0, 0), this.captureContext2D.globalCompositeOperation = "source-over", this.captureCanvas.toDataURL()
                } catch (e) {}
            }, e.prototype.getWebGlConstant = function (e) {
                var t = R[e];
                return t ? t.name : e + ""
            }, e.prototype._setSmoothing = function (e) {
                this.captureContext2D.imageSmoothingEnabled = e, this.captureContext2D.mozImageSmoothingEnabled = e, this.captureContext2D.oImageSmoothingEnabled = e, this.captureContext2D.webkitImageSmoothingEnabled = e, this.captureContext2D.msImageSmoothingEnabled = e
            }, e.captureBaseSize = 64, e.cubeMapFaces = [m.TEXTURE_CUBE_MAP_POSITIVE_X, m.TEXTURE_CUBE_MAP_POSITIVE_Y, m.TEXTURE_CUBE_MAP_POSITIVE_Z, m.TEXTURE_CUBE_MAP_NEGATIVE_X, m.TEXTURE_CUBE_MAP_NEGATIVE_Y, m.TEXTURE_CUBE_MAP_NEGATIVE_Z], e
        }(),
        _t = function () {
            function e(e) {
                this.context = e.context
            }
            return e.prototype.getUboValue = function (t, n, r, a) {
                var o = e.uboTypes[a];
                if (o) {
                    var i = new o.arrayBufferView(r * o.lengthMultiplier),
                        s = this.context,
                        u = s.getIndexedParameter(m.UNIFORM_BUFFER_BINDING.value, t);
                    if (u) {
                        var c = s.getIndexedParameter(m.UNIFORM_BUFFER_START.value, t),
                            E = s.getParameter(m.UNIFORM_BUFFER_BINDING.value);
                        try {
                            s.bindBuffer(m.UNIFORM_BUFFER.value, u), s.getBufferSubData(m.UNIFORM_BUFFER.value, c + n, i)
                        } catch (e) {
                            return
                        }
                        E && s.bindBuffer(m.UNIFORM_BUFFER.value, E)
                    }
                    return Array.prototype.slice.call(i)
                }
            }, e.uboTypes = ((Te = {})[m.BOOL.value] = {
                arrayBufferView: Uint8Array,
                lengthMultiplier: 1
            }, Te[m.BOOL_VEC2.value] = {
                arrayBufferView: Uint8Array,
                lengthMultiplier: 2
            }, Te[m.BOOL_VEC3.value] = {
                arrayBufferView: Uint8Array,
                lengthMultiplier: 3
            }, Te[m.BOOL_VEC4.value] = {
                arrayBufferView: Uint8Array,
                lengthMultiplier: 4
            }, Te[m.INT.value] = {
                arrayBufferView: Int32Array,
                lengthMultiplier: 1
            }, Te[m.INT_VEC2.value] = {
                arrayBufferView: Int32Array,
                lengthMultiplier: 2
            }, Te[m.INT_VEC3.value] = {
                arrayBufferView: Int32Array,
                lengthMultiplier: 3
            }, Te[m.INT_VEC4.value] = {
                arrayBufferView: Int32Array,
                lengthMultiplier: 4
            }, Te[m.UNSIGNED_INT.value] = {
                arrayBufferView: Uint32Array,
                lengthMultiplier: 1
            }, Te[m.UNSIGNED_INT_VEC2.value] = {
                arrayBufferView: Uint32Array,
                lengthMultiplier: 2
            }, Te[m.UNSIGNED_INT_VEC3.value] = {
                arrayBufferView: Uint32Array,
                lengthMultiplier: 3
            }, Te[m.UNSIGNED_INT_VEC4.value] = {
                arrayBufferView: Uint32Array,
                lengthMultiplier: 4
            }, Te[m.FLOAT.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 1
            }, Te[m.FLOAT_VEC2.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 2
            }, Te[m.FLOAT_VEC3.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 3
            }, Te[m.FLOAT_VEC4.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 4
            }, Te[m.FLOAT_MAT2.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 4
            }, Te[m.FLOAT_MAT2x3.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 6
            }, Te[m.FLOAT_MAT2x4.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 8
            }, Te[m.FLOAT_MAT3.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 9
            }, Te[m.FLOAT_MAT3x2.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 6
            }, Te[m.FLOAT_MAT3x4.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 12
            }, Te[m.FLOAT_MAT4.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 16
            }, Te[m.FLOAT_MAT4x2.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 8
            }, Te[m.FLOAT_MAT4x3.value] = {
                arrayBufferView: Float32Array,
                lengthMultiplier: 12
            }, Te[m.SAMPLER_2D.value] = {
                arrayBufferView: Uint8Array,
                lengthMultiplier: 1
            }, Te[m.SAMPLER_CUBE.value] = {
                arrayBufferView: Uint8Array,
                lengthMultiplier: 1
            }, Te), e
        }(),
        pt = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        lt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLBuffer"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        mt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLFramebuffer"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        Tt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLProgram"
                },
                enumerable: !1,
                configurable: !0
            }), t.saveInGlobalStore = function (e) {
                var t = N.getWebGlObjectTag(e);
                t && (this.store[t.id] = e)
            }, t.getFromGlobalStore = function (e) {
                return this.store[e]
            }, t.updateInGlobalStore = function (e, t) {
                if (t) {
                    var n = this.getFromGlobalStore(e);
                    if (n) {
                        var r = N.getWebGlObjectTag(n);
                        r && (N.attachWebGlObjectTag(t, r), this.store[r.id] = t)
                    }
                }
            }, t.store = {}, t
        }(S),
        Rt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLQuery"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        ft = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLRenderbuffer"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        At = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLSampler"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        dt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLShader"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        ht = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLSync"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        Ct = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLTexture"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        Nt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLTransformFeedback"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        St = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLUniformLocation"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        vt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return pt(t, e), Object.defineProperty(t.prototype, "typeName", {
                get: function () {
                    return "WebGLVertexArrayObject"
                },
                enumerable: !1,
                configurable: !0
            }), t
        }(S),
        Ot = function () {
            function e() {}
            return e.getProgramData = function (e, t) {
                for (var n = {
                        LINK_STATUS: e.getProgramParameter(t, m.LINK_STATUS.value),
                        VALIDATE_STATUS: e.getProgramParameter(t, m.VALIDATE_STATUS.value)
                    }, r = e.getAttachedShaders(t), a = new Array(2), o = 0, i = 0, s = r; i < s.length; i++) {
                    var u = s[i],
                        c = this.readShaderFromContext(e, u);
                    o += c.source.length, c.shaderType === m.FRAGMENT_SHADER.name ? a[1] = c : a[0] = c
                }
                return {
                    programStatus: n,
                    shaders: a,
                    length: o
                }
            }, e.readShaderFromContext = function (e, t) {
                var n = e.getShaderSource(t),
                    r = e.getExtension("WEBGL_debug_shaders"),
                    a = r ? r.getTranslatedShaderSource(t) : null,
                    o = e.getShaderParameter(t, m.SHADER_TYPE.value) === m.FRAGMENT_SHADER.value,
                    i = t && t.__SPECTOR_Metadata && t.__SPECTOR_Metadata.name ? t.__SPECTOR_Metadata.name : this.readNameFromShaderSource(n);
                return i || (i = o ? "Fragment" : "Vertex"), {
                    COMPILE_STATUS: e.getShaderParameter(t, m.COMPILE_STATUS.value),
                    shaderType: o ? m.FRAGMENT_SHADER.name : m.VERTEX_SHADER.name,
                    name: i,
                    source: n,
                    translatedSource: a
                }
            }, e.readNameFromShaderSource = function (e) {
                try {
                    var t = "",
                        n = void 0,
                        r = /#define[\s]+SHADER_NAME[\s]+([\S]+)(\n|$)/gi;
                    if (null !== (n = r.exec(e)) && (n.index === r.lastIndex && r.lastIndex++, t = n[1]), "" === t) {
                        var a = /#define[\s]+SHADER_NAME_B64[\s]+([\S]+)(\n|$)/gi;
                        null !== (n = a.exec(e)) && (n.index === a.lastIndex && a.lastIndex++, t = n[1]), t && (t = decodeURIComponent(atob(t)))
                    }
                    return t
                } catch (e) {
                    return null
                }
            }, e
        }(),
        Ft = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        gt = function () {
            return gt = Object.assign || function (e) {
                for (var t, n = 1, r = arguments.length; n < r; n++)
                    for (var a in t = arguments[n]) Object.prototype.hasOwnProperty.call(t, a) && (e[a] = t[a]);
                return e
            }, gt.apply(this, arguments)
        },
        yt = function (e) {
            function t(t) {
                var n = e.call(this, t) || this;
                return n.drawCallTextureInputState = new Et(t), n.drawCallUboInputState = new _t(t), n
            }
            return Ft(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return t.stateName
                },
                enumerable: !1,
                configurable: !0
            }), Object.defineProperty(t.prototype, "requireStartAndStopStates", {
                get: function () {
                    return !1
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getConsumeCommands = function () {
                return _
            }, t.prototype.getChangeCommandsByState = function () {
                return {}
            }, t.prototype.readFromContext = function () {
                var e, n, a = this.context.getParameter(m.CURRENT_PROGRAM.value);
                if (a) {
                    this.currentState.frameBuffer = this.readFrameBufferFromContext();
                    var o = a.__SPECTOR_Object_CustomData ? a.__SPECTOR_Object_CustomData : Ot.getProgramData(this.context, a);
                    if (this.currentState.programStatus = gt({}, o.programStatus), this.currentState.programStatus.program = this.getSpectorData(a), this.currentState.programStatus.RECOMPILABLE = r.isBuildableProgram(a), this.currentState.programStatus.RECOMPILABLE && Tt.saveInGlobalStore(a), this.currentState.shaders = o.shaders, (null === (e = this.lastCommandName) || void 0 === e ? void 0 : e.indexOf("Elements")) >= 0) {
                        var i = this.context.getParameter(this.context.ELEMENT_ARRAY_BUFFER_BINDING);
                        i && (this.currentState.elementArray = {}, this.currentState.elementArray.arrayBuffer = this.getSpectorData(i))
                    }
                    var s = this.context.getProgramParameter(a, m.ACTIVE_ATTRIBUTES.value);
                    this.currentState.attributes = [];
                    for (var u = 0; u < s; u++) {
                        var c = this.readAttributeFromContext(a, u);
                        this.currentState.attributes.push(c)
                    }
                    var E = this.context.getProgramParameter(a, m.ACTIVE_UNIFORMS.value);
                    this.currentState.uniforms = [];
                    var _ = [];
                    for (u = 0; u < E; u++) {
                        _.push(u);
                        var p = this.readUniformFromContext(a, u);
                        this.currentState.uniforms.push(p)
                    }
                    if (this.contextVersion > 1) {
                        var l = this.context.getProgramParameter(a, m.ACTIVE_UNIFORM_BLOCKS.value);
                        for (this.currentState.uniformBlocks = [], u = 0; u < l; u++) {
                            var T = this.readUniformBlockFromContext(a, u);
                            this.currentState.uniformBlocks.push(T)
                        }
                        if (this.readUniformsFromContextIntoState(a, _, this.currentState.uniforms, this.currentState.uniformBlocks), this.context.getParameter(m.TRANSFORM_FEEDBACK_ACTIVE.value)) {
                            var R = this.context.getProgramParameter(a, m.TRANSFORM_FEEDBACK_BUFFER_MODE.value);
                            this.currentState.transformFeedbackMode = this.getWebGlConstant(R), this.currentState.transformFeedbacks = [];
                            var f = this.context.getProgramParameter(a, m.TRANSFORM_FEEDBACK_VARYINGS.value);
                            for (u = 0; u < f; u++) {
                                var A = this.readTransformFeedbackFromContext(a, u);
                                this.currentState.transformFeedbacks.push(A)
                            }
                        }
                    }
                    for (u = 0; u < _.length; u++) {
                        var d = null !== (n = (p = this.currentState.uniforms[u]).value) && void 0 !== n ? n : p.values;
                        if (null != d) {
                            var h = t.samplerTypes[p.typeValue];
                            if (h)
                                if (d.length) {
                                    p.textures = [];
                                    for (var C = 0; C < d.length; C++) p.textures.push(this.readTextureFromContext(d[C].value, h))
                                } else p.texture = this.readTextureFromContext(d, h)
                        }
                        delete p.typeValue
                    }
                }
            }, t.prototype.readFrameBufferFromContext = function () {
                var e = this.context.getParameter(m.FRAMEBUFFER_BINDING.value);
                if (!e) return null;
                var t = {};
                if (t.frameBuffer = this.getSpectorData(e), this.readFrameBufferAttachmentFromContext(m.DEPTH_ATTACHMENT.value) && (t.depthAttachment = this.readFrameBufferAttachmentFromContext(m.DEPTH_ATTACHMENT.value)), this.readFrameBufferAttachmentFromContext(m.STENCIL_ATTACHMENT.value) && (t.stencilAttachment = this.readFrameBufferAttachmentFromContext(m.STENCIL_ATTACHMENT.value)), this.extensions[m.MAX_DRAW_BUFFERS_WEBGL.extensionName]) {
                    t.colorAttachments = [];
                    for (var n = this.context.getParameter(m.MAX_DRAW_BUFFERS_WEBGL.value), r = 0; r < n; r++)(o = this.readFrameBufferAttachmentFromContext(T["COLOR_ATTACHMENT" + r + "_WEBGL"].value)) && t.colorAttachments.push(o)
                } else if (this.contextVersion > 1) {
                    var a = this.context;
                    for (t.colorAttachments = [], n = a.getParameter(m.MAX_DRAW_BUFFERS.value), r = 0; r < n; r++)(o = this.readFrameBufferAttachmentFromContext(T["COLOR_ATTACHMENT" + r].value)) && t.colorAttachments.push(o)
                } else {
                    var o;
                    (o = this.readFrameBufferAttachmentFromContext(T.COLOR_ATTACHMENT0.value)) && (t.colorAttachments = [o])
                }
                return t
            }, t.prototype.readFrameBufferAttachmentFromContext = function (e) {
                var t = m.FRAMEBUFFER.value,
                    n = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE.value);
                if (n !== m.NONE.value) {
                    var r = {},
                        a = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME.value);
                    if (n === m.RENDERBUFFER.value) {
                        if (r.type = "RENDERBUFFER", r.buffer = this.getSpectorData(a), a) {
                            var o = a.__SPECTOR_Object_CustomData;
                            o && (o.internalFormat && (r.internalFormat = this.getWebGlConstant(o.internalFormat)), r.width = o.width, r.height = o.height, r.msaaSamples = o.samples)
                        }
                    } else if (n === m.TEXTURE.value) {
                        r.type = "TEXTURE", r.texture = this.getSpectorData(a), r.textureLevel = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL.value);
                        var i = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE.value);
                        r.textureCubeMapFace = this.getWebGlConstant(i), this.drawCallTextureInputState.appendTextureState(r, a, null, this.fullCapture)
                    }
                    return this.extensions.EXT_sRGB && (r.encoding = this.getWebGlConstant(this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING_EXT.value))), this.contextVersion > 1 && (r.alphaSize = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE.value), r.blueSize = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_BLUE_SIZE.value), r.encoding = this.getWebGlConstant(this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING.value)), r.componentType = this.getWebGlConstant(this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE.value)), r.depthSize = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE.value), r.greenSize = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_GREEN_SIZE.value), r.redSize = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_RED_SIZE.value), r.stencilSize = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE.value), n === m.TEXTURE.value && (r.textureLayer = this.context.getFramebufferAttachmentParameter(t, e, m.FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER.value))), r
                }
            }, t.prototype.readAttributeFromContext = function (e, t) {
                var n = this.context.getActiveAttrib(e, t),
                    r = this.context.getAttribLocation(e, n.name);
                if (-1 === r) return {
                    name: n.name,
                    size: n.size,
                    type: this.getWebGlConstant(n.type),
                    location: -1
                };
                var a = this.context.getVertexAttrib(r, m.CURRENT_VERTEX_ATTRIB.value),
                    o = this.context.getVertexAttrib(r, m.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING.value),
                    i = {
                        name: n.name,
                        size: n.size,
                        type: this.getWebGlConstant(n.type),
                        location: r,
                        offsetPointer: this.context.getVertexAttribOffset(r, m.VERTEX_ATTRIB_ARRAY_POINTER.value),
                        bufferBinding: this.getSpectorData(o),
                        enabled: this.context.getVertexAttrib(r, m.VERTEX_ATTRIB_ARRAY_ENABLED.value),
                        arraySize: this.context.getVertexAttrib(r, m.VERTEX_ATTRIB_ARRAY_SIZE.value),
                        stride: this.context.getVertexAttrib(r, m.VERTEX_ATTRIB_ARRAY_STRIDE.value),
                        arrayType: this.getWebGlConstant(this.context.getVertexAttrib(r, m.VERTEX_ATTRIB_ARRAY_TYPE.value)),
                        normalized: this.context.getVertexAttrib(r, m.VERTEX_ATTRIB_ARRAY_NORMALIZED.value),
                        vertexAttrib: Array.prototype.slice.call(a)
                    };
                return this.extensions[m.VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE.extensionName] ? i.divisor = this.context.getVertexAttrib(r, m.VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE.value) : this.contextVersion > 1 && (i.integer = this.context.getVertexAttrib(r, m.VERTEX_ATTRIB_ARRAY_INTEGER.value), i.divisor = this.context.getVertexAttrib(r, m.VERTEX_ATTRIB_ARRAY_DIVISOR.value)), this.appendBufferCustomData(i, o), i
            }, t.prototype.readUniformFromContext = function (e, t) {
                var n = this.context.getActiveUniform(e, t),
                    r = this.context.getUniformLocation(e, n.name);
                if (r) {
                    if (n.size > 1 && n.name && n.name.indexOf("[0]") === n.name.length - 3) {
                        for (var a = [], o = 0; o < n.size; o++) {
                            var i = this.context.getUniformLocation(e, n.name.replace("[0]", "[" + o + "]"));
                            i && ((s = this.context.getUniform(e, i)).length && (s = Array.prototype.slice.call(s)), a.push({
                                value: s
                            }))
                        }
                        return {
                            name: n.name.replace("[0]", ""),
                            size: n.size,
                            type: this.getWebGlConstant(n.type),
                            typeValue: n.type,
                            location: this.getSpectorData(r),
                            values: a
                        }
                    }
                    var s;
                    return (s = this.context.getUniform(e, r)).length && (s = Array.prototype.slice.call(s)), {
                        name: n.name,
                        size: n.size,
                        type: this.getWebGlConstant(n.type),
                        typeValue: n.type,
                        location: this.getSpectorData(r),
                        value: s
                    }
                }
                return {
                    name: n.name,
                    size: n.size,
                    type: this.getWebGlConstant(n.type),
                    typeValue: n.type
                }
            }, t.prototype.readTextureFromContext = function (e, t) {
                var n = this.context.getParameter(m.ACTIVE_TEXTURE.value);
                this.context.activeTexture(m.TEXTURE0.value + e);
                var r = {
                    magFilter: this.getWebGlConstant(this.context.getTexParameter(t.value, m.TEXTURE_MAG_FILTER.value)),
                    minFilter: this.getWebGlConstant(this.context.getTexParameter(t.value, m.TEXTURE_MIN_FILTER.value)),
                    wrapS: this.getWebGlConstant(this.context.getTexParameter(t.value, m.TEXTURE_WRAP_S.value)),
                    wrapT: this.getWebGlConstant(this.context.getTexParameter(t.value, m.TEXTURE_WRAP_T.value))
                };
                if (this.extensions[m.TEXTURE_MAX_ANISOTROPY_EXT.extensionName] && (r.anisotropy = this.context.getTexParameter(t.value, m.TEXTURE_MAX_ANISOTROPY_EXT.value)), this.contextVersion > 1) {
                    r.baseLevel = this.context.getTexParameter(t.value, m.TEXTURE_BASE_LEVEL.value), r.immutable = this.context.getTexParameter(t.value, m.TEXTURE_IMMUTABLE_FORMAT.value), r.immutableLevels = this.context.getTexParameter(t.value, m.TEXTURE_IMMUTABLE_LEVELS.value), r.maxLevel = this.context.getTexParameter(t.value, m.TEXTURE_MAX_LEVEL.value);
                    var a = this.context.getParameter(m.SAMPLER_BINDING.value);
                    if (a) {
                        r.sampler = this.getSpectorData(a);
                        var o = this.context;
                        r.samplerMaxLod = o.getSamplerParameter(a, m.TEXTURE_MAX_LOD.value), r.samplerMinLod = o.getSamplerParameter(a, m.TEXTURE_MIN_LOD.value), r.samplerCompareFunc = this.getWebGlConstant(o.getSamplerParameter(a, m.TEXTURE_COMPARE_FUNC.value)), r.samplerCompareMode = this.getWebGlConstant(o.getSamplerParameter(a, m.TEXTURE_COMPARE_MODE.value)), r.samplerWrapS = this.getWebGlConstant(o.getSamplerParameter(a, m.TEXTURE_WRAP_S.value)), r.samplerWrapT = this.getWebGlConstant(o.getSamplerParameter(a, m.TEXTURE_WRAP_T.value)), r.samplerWrapR = this.getWebGlConstant(o.getSamplerParameter(a, m.TEXTURE_WRAP_R.value)), r.samplerMagFilter = this.getWebGlConstant(o.getSamplerParameter(a, m.TEXTURE_MAG_FILTER.value)), r.samplerMinFilter = this.getWebGlConstant(o.getSamplerParameter(a, m.TEXTURE_MIN_FILTER.value))
                    } else r.maxLod = this.context.getTexParameter(t.value, m.TEXTURE_MAX_LOD.value), r.minLod = this.context.getTexParameter(t.value, m.TEXTURE_MIN_LOD.value), r.compareFunc = this.getWebGlConstant(this.context.getTexParameter(t.value, m.TEXTURE_COMPARE_FUNC.value)), r.compareMode = this.getWebGlConstant(this.context.getTexParameter(t.value, m.TEXTURE_COMPARE_MODE.value)), r.wrapR = this.getWebGlConstant(this.context.getTexParameter(t.value, m.TEXTURE_WRAP_R.value))
                }
                var i = this.getTextureStorage(t);
                if (i) {
                    var s = this.quickCapture ? null : t;
                    this.drawCallTextureInputState.appendTextureState(r, i, s, this.fullCapture)
                }
                return this.context.activeTexture(n), r
            }, t.prototype.getTextureStorage = function (e) {
                return e === m.TEXTURE_2D ? this.context.getParameter(m.TEXTURE_BINDING_2D.value) : e === m.TEXTURE_CUBE_MAP ? this.context.getParameter(m.TEXTURE_BINDING_CUBE_MAP.value) : e === m.TEXTURE_3D ? this.context.getParameter(m.TEXTURE_BINDING_3D.value) : e === m.TEXTURE_2D_ARRAY ? this.context.getParameter(m.TEXTURE_BINDING_2D_ARRAY.value) : void 0
            }, t.prototype.readUniformsFromContextIntoState = function (e, t, n, r) {
                for (var a = this.context, o = a.getActiveUniforms(e, t, m.UNIFORM_TYPE.value), i = a.getActiveUniforms(e, t, m.UNIFORM_SIZE.value), s = a.getActiveUniforms(e, t, m.UNIFORM_BLOCK_INDEX.value), u = a.getActiveUniforms(e, t, m.UNIFORM_OFFSET.value), c = a.getActiveUniforms(e, t, m.UNIFORM_ARRAY_STRIDE.value), E = a.getActiveUniforms(e, t, m.UNIFORM_MATRIX_STRIDE.value), _ = a.getActiveUniforms(e, t, m.UNIFORM_IS_ROW_MAJOR.value), p = 0; p < t.length; p++) {
                    var l = n[p];
                    if (l.type = this.getWebGlConstant(o[p]), l.size = i[p], l.blockIndice = s[p], l.blockIndice > -1 && (l.blockName = a.getActiveUniformBlockName(e, l.blockIndice)), l.offset = u[p], l.arrayStride = c[p], l.matrixStride = E[p], l.rowMajor = _[p], l.blockIndice > -1) {
                        var T = r[s[p]].bindingPoint;
                        l.value = this.drawCallUboInputState.getUboValue(T, l.offset, l.size, o[p])
                    }
                }
            }, t.prototype.readTransformFeedbackFromContext = function (e, t) {
                var n = this.context,
                    r = n.getTransformFeedbackVarying(e, t),
                    a = n.getIndexedParameter(m.TRANSFORM_FEEDBACK_BUFFER_BINDING.value, t),
                    o = {
                        name: r.name,
                        size: r.size,
                        type: this.getWebGlConstant(r.type),
                        buffer: this.getSpectorData(a),
                        bufferSize: n.getIndexedParameter(m.TRANSFORM_FEEDBACK_BUFFER_SIZE.value, t),
                        bufferStart: n.getIndexedParameter(m.TRANSFORM_FEEDBACK_BUFFER_START.value, t)
                    };
                return this.appendBufferCustomData(o, a), o
            }, t.prototype.readUniformBlockFromContext = function (e, t) {
                var n = this.context,
                    r = n.getActiveUniformBlockParameter(e, t, m.UNIFORM_BLOCK_BINDING.value),
                    a = n.getIndexedParameter(m.UNIFORM_BUFFER_BINDING.value, r),
                    o = {
                        name: n.getActiveUniformBlockName(e, t),
                        bindingPoint: r,
                        size: n.getActiveUniformBlockParameter(e, t, m.UNIFORM_BLOCK_DATA_SIZE.value),
                        activeUniformCount: n.getActiveUniformBlockParameter(e, t, m.UNIFORM_BLOCK_ACTIVE_UNIFORMS.value),
                        vertex: n.getActiveUniformBlockParameter(e, t, m.UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER.value),
                        fragment: n.getActiveUniformBlockParameter(e, t, m.UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER.value),
                        buffer: this.getSpectorData(a)
                    };
                return this.appendBufferCustomData(o, a), o
            }, t.prototype.appendBufferCustomData = function (e, t) {
                if (t) {
                    var n = t.__SPECTOR_Object_CustomData;
                    n && (n.usage && (e.bufferUsage = this.getWebGlConstant(n.usage)), e.bufferLength = n.length, n.offset && (e.bufferOffset = n.offset), n.sourceLength && (e.bufferSourceLength = n.sourceLength))
                }
            }, t.prototype.getWebGlConstant = function (e) {
                var t = R[e];
                return t ? t.name : e
            }, t.stateName = "DrawCall", t.samplerTypes = ((Re = {})[m.SAMPLER_2D.value] = m.TEXTURE_2D, Re[m.SAMPLER_CUBE.value] = m.TEXTURE_CUBE_MAP, Re[m.SAMPLER_3D.value] = m.TEXTURE_3D, Re[m.SAMPLER_2D_SHADOW.value] = m.TEXTURE_2D, Re[m.SAMPLER_2D_ARRAY.value] = m.TEXTURE_2D_ARRAY, Re[m.SAMPLER_2D_ARRAY_SHADOW.value] = m.TEXTURE_2D_ARRAY, Re[m.SAMPLER_CUBE_SHADOW.value] = m.TEXTURE_CUBE_MAP, Re[m.INT_SAMPLER_2D.value] = m.TEXTURE_2D, Re[m.INT_SAMPLER_3D.value] = m.TEXTURE_3D, Re[m.INT_SAMPLER_CUBE.value] = m.TEXTURE_CUBE_MAP, Re[m.INT_SAMPLER_2D_ARRAY.value] = m.TEXTURE_2D_ARRAY, Re[m.UNSIGNED_INT_SAMPLER_2D.value] = m.TEXTURE_2D, Re[m.UNSIGNED_INT_SAMPLER_3D.value] = m.TEXTURE_3D, Re[m.UNSIGNED_INT_SAMPLER_CUBE.value] = m.TEXTURE_CUBE_MAP, Re[m.UNSIGNED_INT_SAMPLER_2D_ARRAY.value] = m.TEXTURE_2D_ARRAY, Re), t
        }(Me),
        It = function () {
            function e(e) {
                this.contextInformation = e, this.stateTrackers = [], this.onCommandCapturedCallbacks = {}, this.initStateTrackers()
            }
            return e.prototype.startCapture = function (e, t, n) {
                for (var r = 0, a = this.stateTrackers; r < a.length; r++) {
                    var o = a[r],
                        i = o.startCapture(!0, t, n);
                    o.requireStartAndStopStates && (e.initState[o.stateName] = i)
                }
            }, e.prototype.stopCapture = function (e) {
                for (var t = 0, n = this.stateTrackers; t < n.length; t++) {
                    var r = n[t],
                        a = r.stopCapture();
                    r.requireStartAndStopStates && (e.endState[r.stateName] = a)
                }
            }, e.prototype.captureState = function (e) {
                var t = this.onCommandCapturedCallbacks[e.name];
                if (t)
                    for (var n = 0, r = t; n < r.length; n++)(0, r[n])(e)
            }, e.prototype.initStateTrackers = function () {
                this.stateTrackers.push(new De(this.contextInformation), new xe(this.contextInformation), new Xe(this.contextInformation), new Ve(this.contextInformation), new je(this.contextInformation), new Ke(this.contextInformation), new Ze(this.contextInformation), new qe(this.contextInformation), new Je(this.contextInformation), new et(this.contextInformation), new rt(this.contextInformation), new ot(this.contextInformation), new ut(this.contextInformation), new yt(this.contextInformation));
                for (var e = 0, t = this.stateTrackers; e < t.length; e++) t[e].registerCallbacks(this.onCommandCapturedCallbacks)
            }, e
        }(),
        Bt = function () {
            function e(t) {
                this.options = t, this.createCommandNames = this.getCreateCommandNames(), this.updateCommandNames = this.getUpdateCommandNames(), this.deleteCommandNames = this.getDeleteCommandNames(), this.startTime = s.now, this.memoryPerSecond = {}, this.totalMemory = 0, this.frameMemory = 0, this.capturing = !1, e.initializeByteSizeFormat()
            }
            return e.initializeByteSizeFormat = function () {
                var e;
                this.byteSizePerInternalFormat || (this.byteSizePerInternalFormat = ((e = {})[m.R8.value] = 1, e[m.R16F.value] = 2, e[m.R32F.value] = 4, e[m.R8UI.value] = 1, e[m.RG8.value] = 2, e[m.RG16F.value] = 4, e[m.RG32F.value] = 8, e[m.ALPHA.value] = 1, e[m.RGB.value] = 3, e[m.RGBA.value] = 4, e[m.LUMINANCE.value] = 1, e[m.LUMINANCE_ALPHA.value] = 2, e[m.DEPTH_COMPONENT.value] = 1, e[m.DEPTH_STENCIL.value] = 2, e[m.SRGB_EXT.value] = 3, e[m.SRGB_ALPHA_EXT.value] = 4, e[m.RGB8.value] = 3, e[m.SRGB8.value] = 3, e[m.RGB565.value] = 2, e[m.R11F_G11F_B10F.value] = 4, e[m.RGB9_E5.value] = 2, e[m.RGB16F.value] = 6, e[m.RGB32F.value] = 12, e[m.RGB8UI.value] = 3, e[m.RGBA8.value] = 4, e[m.RGB5_A1.value] = 2, e[m.RGBA16F.value] = 8, e[m.RGBA32F.value] = 16, e[m.RGBA8UI.value] = 4, e[m.COMPRESSED_R11_EAC.value] = 4, e[m.COMPRESSED_SIGNED_R11_EAC.value] = 4, e[m.COMPRESSED_RG11_EAC.value] = 4, e[m.COMPRESSED_SIGNED_RG11_EAC.value] = 4, e[m.COMPRESSED_RGB8_ETC2.value] = 4, e[m.COMPRESSED_RGBA8_ETC2_EAC.value] = 4, e[m.COMPRESSED_SRGB8_ETC2.value] = 4, e[m.COMPRESSED_SRGB8_ALPHA8_ETC2_EAC.value] = 4, e[m.COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2.value] = 4, e[m.COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2.value] = 4, e[m.COMPRESSED_RGB_S3TC_DXT1_EXT.value] = .5, e[m.COMPRESSED_RGBA_S3TC_DXT3_EXT.value] = 1, e[m.COMPRESSED_RGBA_S3TC_DXT5_EXT.value] = 1, e[m.COMPRESSED_RGB_PVRTC_4BPPV1_IMG.value] = .5, e[m.COMPRESSED_RGBA_PVRTC_4BPPV1_IMG.value] = .5, e[m.COMPRESSED_RGB_PVRTC_2BPPV1_IMG.value] = .25, e[m.COMPRESSED_RGBA_PVRTC_2BPPV1_IMG.value] = .25, e[m.COMPRESSED_RGB_ETC1_WEBGL.value] = .5, e[m.COMPRESSED_RGB_ATC_WEBGL.value] = .5, e[m.COMPRESSED_RGBA_ATC_EXPLICIT_ALPHA_WEBGL.value] = 1, e[m.COMPRESSED_RGBA_ATC_INTERPOLATED_ALPHA_WEBGL.value] = 1, e))
            }, e.prototype.registerCallbacks = function (e) {
                for (var t = 0, n = this.createCommandNames; t < n.length; t++) e[s = n[t]] = e[s] || [], e[s].push(this.createWithoutSideEffects.bind(this));
                for (var r = 0, a = this.updateCommandNames; r < a.length; r++) e[s = a[r]] = e[s] || [], e[s].push(this.updateWithoutSideEffects.bind(this));
                for (var o = 0, i = this.deleteCommandNames; o < i.length; o++) {
                    var s;
                    e[s = i[o]] = e[s] || [], e[s].push(this.deleteWithoutSideEffects.bind(this))
                }
            }, e.prototype.startCapture = function () {
                this.frameMemory = 0, this.capturing = !0
            }, e.prototype.stopCapture = function () {
                this.frameMemory = 0, this.capturing = !1
            }, e.prototype.appendRecordedInformation = function (e) {
                e.frameMemory[this.objectName] = this.frameMemory, e.memory[this.objectName] = this.memoryPerSecond
            }, e.prototype.create = function (e) {}, e.prototype.createWithoutSideEffects = function (e) {
                this.options.toggleCapture(!1), this.create(e), this.options.toggleCapture(!0)
            }, e.prototype.updateWithoutSideEffects = function (e) {
                if (e && 0 !== e.arguments.length) {
                    this.options.toggleCapture(!1);
                    var t = e.arguments[0],
                        n = this.getBoundInstance(t);
                    if (n)
                        if (N.getWebGlObjectTag(n)) {
                            var r = this.getWebGlConstant(t),
                                a = this.update(e, r, n);
                            this.changeMemorySize(a), this.options.toggleCapture(!0)
                        } else this.options.toggleCapture(!0);
                    else this.options.toggleCapture(!0)
                }
            }, e.prototype.deleteWithoutSideEffects = function (e) {
                if (e && e.arguments && !(e.arguments.length < 1)) {
                    var t = e.arguments[0];
                    if (t) {
                        this.options.toggleCapture(!1);
                        var n = this.delete(t);
                        this.changeMemorySize(-n), this.options.toggleCapture(!0)
                    }
                }
            }, e.prototype.changeMemorySize = function (e) {
                this.totalMemory += e, this.capturing && (this.frameMemory += e);
                var t = s.now - this.startTime,
                    n = Math.round(t / 1e3);
                this.memoryPerSecond[n] = this.totalMemory
            }, e.prototype.getWebGlConstant = function (e) {
                var t = R[e];
                return t ? t.name : e + ""
            }, e.prototype.getByteSizeForInternalFormat = function (t) {
                return e.byteSizePerInternalFormat[t] || 4
            }, e
        }(),
        Pt = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Mt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Pt(t, e), Object.defineProperty(t.prototype, "objectName", {
                get: function () {
                    return "Buffer"
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getCreateCommandNames = function () {
                return ["createBuffer"]
            }, t.prototype.getUpdateCommandNames = function () {
                return ["bufferData"]
            }, t.prototype.getDeleteCommandNames = function () {
                return ["deleteBuffer"]
            }, t.prototype.getBoundInstance = function (e) {
                var t = this.options.context;
                return e === m.ARRAY_BUFFER.value ? t.getParameter(m.ARRAY_BUFFER_BINDING.value) : e === m.ELEMENT_ARRAY_BUFFER.value ? t.getParameter(m.ELEMENT_ARRAY_BUFFER_BINDING.value) : e === m.COPY_READ_BUFFER.value ? t.getParameter(m.COPY_READ_BUFFER_BINDING.value) : e === m.COPY_WRITE_BUFFER.value ? t.getParameter(m.COPY_WRITE_BUFFER_BINDING.value) : e === m.TRANSFORM_FEEDBACK_BUFFER.value ? t.getParameter(m.TRANSFORM_FEEDBACK_BUFFER_BINDING.value) : e === m.UNIFORM_BUFFER.value ? t.getParameter(m.UNIFORM_BUFFER_BINDING.value) : e === m.PIXEL_PACK_BUFFER.value ? t.getParameter(m.PIXEL_PACK_BUFFER_BINDING.value) : e === m.PIXEL_UNPACK_BUFFER.value ? t.getParameter(m.PIXEL_UNPACK_BUFFER_BINDING.value) : void 0
            }, t.prototype.delete = function (e) {
                var t = e.__SPECTOR_Object_CustomData;
                return t ? t.length : 0
            }, t.prototype.update = function (e, t, n) {
                var r = this.getCustomData(t, e);
                if (!r) return 0;
                var a = n.__SPECTOR_Object_CustomData ? n.__SPECTOR_Object_CustomData.length : 0;
                return n.__SPECTOR_Object_CustomData = r, r.length - a
            }, t.prototype.getCustomData = function (e, t) {
                var n = this.getLength(t);
                return t.arguments.length >= 4 ? {
                    target: e,
                    length: n,
                    usage: t.arguments[2],
                    offset: t.arguments[3],
                    sourceLength: t.arguments[1] ? t.arguments[1].length : -1
                } : 3 === t.arguments.length ? {
                    target: e,
                    length: n,
                    usage: t.arguments[2]
                } : void 0
            }, t.prototype.getLength = function (e) {
                var t = -1,
                    n = 0;
                return 5 === e.arguments.length && (t = e.arguments[4], n = e.arguments[3]), t <= 0 && (t = "number" == typeof e.arguments[1] ? e.arguments[1] : e.arguments[1] && (e.arguments[1].byteLength || e.arguments[1].length) || 0), t - n
            }, t
        }(Bt),
        Lt = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        bt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Lt(t, e), Object.defineProperty(t.prototype, "objectName", {
                get: function () {
                    return "Renderbuffer"
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getCreateCommandNames = function () {
                return ["createRenderbuffer"]
            }, t.prototype.getUpdateCommandNames = function () {
                return ["renderbufferStorage", "renderbufferStorageMultisample"]
            }, t.prototype.getDeleteCommandNames = function () {
                return ["deleteRenderbuffer"]
            }, t.prototype.getBoundInstance = function (e) {
                var t = this.options.context;
                if (e === m.RENDERBUFFER.value) return t.getParameter(m.RENDERBUFFER_BINDING.value)
            }, t.prototype.delete = function (e) {
                var t = e.__SPECTOR_Object_CustomData;
                return t ? t.length : 0
            }, t.prototype.update = function (e, t, n) {
                var r = this.getCustomData(e, t);
                if (!r) return 0;
                var a = n.__SPECTOR_Object_CustomData ? n.__SPECTOR_Object_CustomData.length : 0;
                return r.length = r.width * r.height * this.getByteSizeForInternalFormat(r.internalFormat), n.__SPECTOR_Object_CustomData = r, r.length - a
            }, t.prototype.getCustomData = function (e, t) {
                return 4 === e.arguments.length ? {
                    target: t,
                    internalFormat: e.arguments[1],
                    width: e.arguments[2],
                    height: e.arguments[3],
                    length: 0,
                    samples: 0
                } : {
                    target: t,
                    internalFormat: e.arguments[2],
                    width: e.arguments[3],
                    height: e.arguments[4],
                    length: 0,
                    samples: e.arguments[1]
                }
            }, t
        }(Bt),
        Ut = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Dt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Ut(t, e), Object.defineProperty(t.prototype, "objectName", {
                get: function () {
                    return "Texture2d"
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getCreateCommandNames = function () {
                return ["createTexture"]
            }, t.prototype.getUpdateCommandNames = function () {
                return ["texImage2D", "compressedTexImage2D", "texStorage2D"]
            }, t.prototype.getDeleteCommandNames = function () {
                return ["deleteTexture"]
            }, t.prototype.getBoundInstance = function (e) {
                var t = this.options.context;
                return e === m.TEXTURE_2D.value ? t.getParameter(m.TEXTURE_BINDING_2D.value) : e === m.TEXTURE_CUBE_MAP_POSITIVE_X.value || e === m.TEXTURE_CUBE_MAP_POSITIVE_Y.value || e === m.TEXTURE_CUBE_MAP_POSITIVE_Z.value || e === m.TEXTURE_CUBE_MAP_NEGATIVE_X.value || e === m.TEXTURE_CUBE_MAP_NEGATIVE_Y.value || e === m.TEXTURE_CUBE_MAP_NEGATIVE_Z.value ? t.getParameter(m.TEXTURE_BINDING_CUBE_MAP.value) : void 0
            }, t.prototype.delete = function (e) {
                var t = e.__SPECTOR_Object_CustomData;
                return t ? t.target === m.TEXTURE_2D_ARRAY.name || t.target === m.TEXTURE_3D.name ? 0 : t.length : 0
            }, t.prototype.update = function (e, t, n) {
                var r = this.getCustomData(e, t, n);
                if (!r) return 0;
                var a = n.__SPECTOR_Object_CustomData ? n.__SPECTOR_Object_CustomData.length : 0;
                if (r.isCompressed) {
                    if (e.arguments.length >= 7) {
                        var o = e.arguments[6];
                        r.length = "number" == typeof o ? o : null == o ? void 0 : o.byteLength
                    }
                } else {
                    var i = "TEXTURE_2D" === t ? 1 : 6,
                        s = r.internalFormat;
                    s === m.RGBA.value && (r.type === m.FLOAT.value && (s = m.RGBA32F.value), r.type === m.HALF_FLOAT_OES.value && (s = m.RGBA16F.value)), r.length = r.width * r.height * i * this.getByteSizeForInternalFormat(s)
                }
                return r.length = 0 | r.length, n.__SPECTOR_Object_CustomData = r, r.length - a
            }, t.prototype.getCustomData = function (e, t, n) {
                return "texImage2D" === e.name ? this.getTexImage2DCustomData(e, t, n) : "compressedTexImage2D" === e.name ? this.getCompressedTexImage2DCustomData(e, t, n) : "texStorage2D" === e.name ? this.getTexStorage2DCustomData(e, t, n) : void 0
            }, t.prototype.getTexStorage2DCustomData = function (e, t, n) {
                var r;
                return 5 === e.arguments.length && (r = {
                    target: t,
                    internalFormat: e.arguments[2],
                    width: e.arguments[3],
                    height: e.arguments[4],
                    length: 0,
                    isCompressed: !1
                }), r
            }, t.prototype.getCompressedTexImage2DCustomData = function (e, t, n) {
                var r;
                if (0 === e.arguments[1]) return e.arguments.length >= 7 && (r = {
                    target: t,
                    internalFormat: e.arguments[2],
                    width: e.arguments[3],
                    height: e.arguments[4],
                    length: 0,
                    isCompressed: !0
                }), r
            }, t.prototype.getTexImage2DCustomData = function (e, t, n) {
                var r;
                if (0 === e.arguments[1]) return e.arguments.length >= 8 ? r = {
                    target: t,
                    internalFormat: e.arguments[2],
                    width: e.arguments[3],
                    height: e.arguments[4],
                    format: e.arguments[6],
                    type: e.arguments[7],
                    length: 0,
                    isCompressed: !1
                } : 6 === e.arguments.length && (r = {
                    target: t,
                    internalFormat: e.arguments[2],
                    width: e.arguments[5].width,
                    height: e.arguments[5].height,
                    format: e.arguments[3],
                    type: e.arguments[4],
                    length: 0,
                    isCompressed: !1
                }), r
            }, t
        }(Bt),
        Gt = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        xt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return Gt(t, e), Object.defineProperty(t.prototype, "objectName", {
                get: function () {
                    return "Texture3d"
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getCreateCommandNames = function () {
                return ["createTexture"]
            }, t.prototype.getUpdateCommandNames = function () {
                return ["texImage3D", "compressedTexImage3D", "texStorage3D"]
            }, t.prototype.getDeleteCommandNames = function () {
                return ["deleteTexture"]
            }, t.prototype.getBoundInstance = function (e) {
                var t = this.options.context;
                return e === m.TEXTURE_2D_ARRAY.value ? t.getParameter(m.TEXTURE_BINDING_2D_ARRAY.value) : e === m.TEXTURE_3D.value ? t.getParameter(m.TEXTURE_BINDING_3D.value) : void 0
            }, t.prototype.delete = function (e) {
                var t = e.__SPECTOR_Object_CustomData;
                return t ? t.target !== m.TEXTURE_2D_ARRAY.name && t.target !== m.TEXTURE_3D.name ? 0 : t.length : 0
            }, t.prototype.update = function (e, t, n) {
                if (e.arguments.length >= 2 && 0 !== e.arguments[1]) return 0;
                var r = this.getCustomData(e, t, n);
                if (!r) return 0;
                var a = n.__SPECTOR_Object_CustomData ? n.__SPECTOR_Object_CustomData.length : 0;
                if (r.isCompressed) {
                    if (e.arguments.length >= 7) {
                        var o = e.arguments[6];
                        r.length = "number" == typeof o ? o : null == o ? void 0 : o.byteLength
                    }
                } else r.length = r.width * r.height * r.depth * this.getByteSizeForInternalFormat(r.internalFormat);
                return r.length = 0 | r.length, n.__SPECTOR_Object_CustomData = r, r.length - a
            }, t.prototype.getCustomData = function (e, t, n) {
                return "texImage3D" === e.name ? this.getTexImage3DCustomData(e, t, n) : "compressedTexImage3D" === e.name ? this.getCompressedTexImage3DCustomData(e, t, n) : "texStorage3D" === e.name ? this.getTexStorage3DCustomData(e, t, n) : void 0
            }, t.prototype.getTexStorage3DCustomData = function (e, t, n) {
                var r;
                return 6 === e.arguments.length && (r = {
                    target: t,
                    internalFormat: e.arguments[2],
                    width: e.arguments[3],
                    height: e.arguments[4],
                    depth: e.arguments[5],
                    length: 0,
                    isCompressed: !1
                }), r
            }, t.prototype.getCompressedTexImage3DCustomData = function (e, t, n) {
                var r;
                if (0 === e.arguments[1]) return e.arguments.length >= 8 && (r = {
                    target: t,
                    internalFormat: e.arguments[2],
                    width: e.arguments[3],
                    height: e.arguments[4],
                    depth: e.arguments[5],
                    length: 0,
                    isCompressed: !0
                }), r
            }, t.prototype.getTexImage3DCustomData = function (e, t, n) {
                var r;
                if (0 === e.arguments[1]) return e.arguments.length >= 9 && (r = {
                    target: t,
                    internalFormat: e.arguments[2],
                    width: e.arguments[3],
                    height: e.arguments[4],
                    depth: e.arguments[5],
                    format: e.arguments[7],
                    type: e.arguments[8],
                    length: 0,
                    isCompressed: !1
                }), r
            }, t
        }(Bt),
        wt = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Xt = function (e) {
            function t() {
                return null !== e && e.apply(this, arguments) || this
            }
            return wt(t, e), Object.defineProperty(t.prototype, "objectName", {
                get: function () {
                    return "Program"
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getCreateCommandNames = function () {
                return ["createProgram"]
            }, t.prototype.getUpdateCommandNames = function () {
                return ["linkProgram"]
            }, t.prototype.getDeleteCommandNames = function () {
                return ["deleteProgram"]
            }, t.prototype.getBoundInstance = function (e) {
                return e
            }, t.prototype.delete = function (e) {
                var t = e.__SPECTOR_Object_CustomData;
                return t ? t.length : 0
            }, t.prototype.update = function (e, t, n) {
                if (e.arguments.length >= 1 && !e.arguments[0]) return 0;
                var r = this.getCustomData(n);
                if (!r) return 0;
                var a = n.__SPECTOR_Object_CustomData ? n.__SPECTOR_Object_CustomData.length : 0;
                return n.__SPECTOR_Object_CustomData = r, r.length - a
            }, t.prototype.getCustomData = function (e) {
                var t = this.options.context;
                return Ot.getProgramData(t, e)
            }, t
        }(Bt),
        Wt = function () {
            function e(e) {
                this.contextInformation = e, this.onCommandCallbacks = {}, this.recorders = [], this.initRecorders()
            }
            return e.prototype.recordCommand = function (e) {
                var t = this.onCommandCallbacks[e.name];
                if (t)
                    for (var n = 0, r = t; n < r.length; n++)(0, r[n])(e)
            }, e.prototype.startCapture = function () {
                for (var e = 0, t = this.recorders; e < t.length; e++) t[e].startCapture()
            }, e.prototype.stopCapture = function () {
                for (var e = 0, t = this.recorders; e < t.length; e++) t[e].stopCapture()
            }, e.prototype.appendRecordedInformation = function (e) {
                for (var t = 0, n = this.recorders; t < n.length; t++) n[t].appendRecordedInformation(e)
            }, e.prototype.initRecorders = function () {
                this.recorders.push(new Mt(this.contextInformation), new bt(this.contextInformation), new Dt(this.contextInformation), new xt(this.contextInformation), new Xt(this.contextInformation));
                for (var e = 0, t = this.recorders; e < t.length; e++) t[e].registerCallbacks(this.onCommandCallbacks)
            }, e
        }(),
        Vt = function () {
            function e(e) {
                this.contextInformation = e, this.webGlObjects = [], this.initWebglObjects()
            }
            return e.prototype.tagWebGlObjects = function (e) {
                for (var t = 0, n = this.webGlObjects; t < n.length; t++) {
                    for (var r = n[t], a = 0; a < e.arguments.length; a++) {
                        var o = e.arguments[a];
                        if (r.tagWebGlObject(o)) break
                    }
                    if (r.tagWebGlObject(e.result)) break
                }
            }, e.prototype.tagWebGlObject = function (e) {
                for (var t = 0, n = this.webGlObjects; t < n.length; t++) {
                    var r = n[t].tagWebGlObject(e);
                    if (r) return r
                }
            }, e.prototype.initWebglObjects = function () {
                this.webGlObjects.push(new lt, new mt, new Tt, new Rt, new ft, new At, new ht, new Ct, new Nt, new St, new vt, new dt)
            }, e
        }(),
        Ht = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        jt = function (e) {
            function t(t) {
                var n = e.call(this, t) || this;
                return n.extensionDefinition = [
                    [{
                        name: "ANGLE_instanced_arrays",
                        description: ""
                    }, {
                        name: "EXT_blend_minmax",
                        description: ""
                    }, {
                        name: "EXT_color_buffer_float",
                        description: ""
                    }, {
                        name: "EXT_color_buffer_half_float",
                        description: ""
                    }, {
                        name: "EXT_disjoint_timer_query",
                        description: ""
                    }, {
                        name: "EXT_frag_depth",
                        description: ""
                    }, {
                        name: "EXT_sRGB",
                        description: ""
                    }, {
                        name: "EXT_shader_texture_lod",
                        description: ""
                    }, {
                        name: "EXT_texture_filter_anisotropic",
                        description: ""
                    }, {
                        name: "OES_element_index_uint",
                        description: ""
                    }, {
                        name: "OES_standard_derivatives",
                        description: ""
                    }, {
                        name: "OES_texture_float",
                        description: ""
                    }, {
                        name: "OES_texture_float_linear",
                        description: ""
                    }, {
                        name: "OES_texture_half_float",
                        description: ""
                    }, {
                        name: "OES_texture_half_float_linear",
                        description: ""
                    }, {
                        name: "OES_vertex_array_object",
                        description: ""
                    }, {
                        name: "WEBGL_color_buffer_float",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_astc",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_atc",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_etc",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_etc1",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_pvrtc",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_s3tc",
                        description: ""
                    }, {
                        name: "WEBGL_depth_texture",
                        description: ""
                    }, {
                        name: "WEBGL_draw_buffers",
                        description: ""
                    }],
                    [{
                        name: "EXT_color_buffer_float",
                        description: ""
                    }, {
                        name: "EXT_disjoint_timer_query",
                        description: ""
                    }, {
                        name: "EXT_disjoint_timer_query_webgl2",
                        description: ""
                    }, {
                        name: "EXT_texture_filter_anisotropic",
                        description: ""
                    }, {
                        name: "OES_texture_float_linear",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_astc",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_atc",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_etc",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_etc1",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_pvrtc",
                        description: ""
                    }, {
                        name: "WEBGL_compressed_texture_s3tc",
                        description: ""
                    }, {
                        name: "WEBGL_multi_draw_instanced_base_vertex_base_instance",
                        description: ""
                    }]
                ], n.currentState = n.startCapture(!0, n.quickCapture, n.fullCapture), n
            }
            return Ht(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return "Extensions"
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getExtensions = function () {
                return this.extensions
            }, t.prototype.readFromContext = function () {
                for (var e = 0, t = 1 === this.contextVersion ? this.extensionDefinition[0] : this.extensionDefinition[1]; e < t.length; e++) {
                    var n = t[e],
                        r = this.context.getExtension(n.name);
                    r ? (this.currentState[n.name] = !0, this.extensions[n.name] = r) : this.currentState[n.name] = !1
                }
            }, t
        }(Me),
        Yt = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Kt = function (e) {
            function t(t) {
                var n = e.call(this, t) || this;
                return n.currentState = n.startCapture(!0, n.quickCapture, n.fullCapture), n
            }
            return Yt(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return "CompressedTextures"
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.COMPRESSED_TEXTURE_FORMATS
                }]
            }, t.prototype.stringifyParameterValue = function (e, t) {
                for (var n = [], r = 0, a = e; r < a.length; r++) {
                    var o = a[r];
                    n.push(m.stringifyWebGlConstant(o, "getParameter"))
                }
                return n
            }, t
        }(be),
        kt = function () {
            var e = function (t, n) {
                return e = Object.setPrototypeOf || {
                    __proto__: []
                }
                instanceof Array && function (e, t) {
                    e.__proto__ = t
                } || function (e, t) {
                    for (var n in t) Object.prototype.hasOwnProperty.call(t, n) && (e[n] = t[n])
                }, e(t, n)
            };
            return function (t, n) {
                if ("function" != typeof n && null !== n) throw new TypeError("Class extends value " + String(n) + " is not a constructor or null");

                function r() {
                    this.constructor = t
                }
                e(t, n), t.prototype = null === n ? Object.create(n) : (r.prototype = n.prototype, new r)
            }
        }(),
        Zt = function (e) {
            function t(t) {
                var n = e.call(this, t) || this;
                return n.currentState = n.startCapture(!0, n.quickCapture, n.fullCapture), n
            }
            return kt(t, e), Object.defineProperty(t.prototype, "stateName", {
                get: function () {
                    return "Capabilities"
                },
                enumerable: !1,
                configurable: !0
            }), t.prototype.getWebgl1Parameters = function () {
                return [{
                    constant: m.RENDERER
                }, {
                    constant: m.VENDOR
                }, {
                    constant: m.VERSION
                }, {
                    constant: m.SHADING_LANGUAGE_VERSION
                }, {
                    constant: m.SAMPLES
                }, {
                    constant: m.SAMPLE_BUFFERS
                }, {
                    constant: m.RED_BITS
                }, {
                    constant: m.GREEN_BITS
                }, {
                    constant: m.BLUE_BITS
                }, {
                    constant: m.ALPHA_BITS
                }, {
                    constant: m.DEPTH_BITS
                }, {
                    constant: m.STENCIL_BITS
                }, {
                    constant: m.SUBPIXEL_BITS
                }, {
                    constant: m.LINE_WIDTH
                }, {
                    constant: m.ALIASED_LINE_WIDTH_RANGE
                }, {
                    constant: m.ALIASED_POINT_SIZE_RANGE
                }, {
                    constant: m.IMPLEMENTATION_COLOR_READ_FORMAT,
                    returnType: 20
                }, {
                    constant: m.IMPLEMENTATION_COLOR_READ_TYPE,
                    returnType: 20
                }, {
                    constant: m.MAX_COMBINED_TEXTURE_IMAGE_UNITS
                }, {
                    constant: m.MAX_CUBE_MAP_TEXTURE_SIZE
                }, {
                    constant: m.MAX_FRAGMENT_UNIFORM_VECTORS
                }, {
                    constant: m.MAX_RENDERBUFFER_SIZE
                }, {
                    constant: m.MAX_TEXTURE_IMAGE_UNITS
                }, {
                    constant: m.MAX_TEXTURE_SIZE
                }, {
                    constant: m.MAX_VARYING_VECTORS
                }, {
                    constant: m.MAX_VERTEX_ATTRIBS
                }, {
                    constant: m.MAX_VERTEX_TEXTURE_IMAGE_UNITS
                }, {
                    constant: m.MAX_VERTEX_UNIFORM_VECTORS
                }, {
                    constant: m.MAX_VIEWPORT_DIMS
                }, {
                    constant: m.MAX_TEXTURE_MAX_ANISOTROPY_EXT
                }, {
                    constant: m.MAX_COLOR_ATTACHMENTS_WEBGL
                }, {
                    constant: m.MAX_DRAW_BUFFERS_WEBGL
                }]
            }, t.prototype.getWebgl2Parameters = function () {
                return [{
                    constant: m.MAX_3D_TEXTURE_SIZE
                }, {
                    constant: m.MAX_ARRAY_TEXTURE_LAYERS
                }, {
                    constant: m.MAX_CLIENT_WAIT_TIMEOUT_WEBGL
                }, {
                    constant: m.MAX_COLOR_ATTACHMENTS
                }, {
                    constant: m.MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS
                }, {
                    constant: m.MAX_COMBINED_UNIFORM_BLOCKS
                }, {
                    constant: m.MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS
                }, {
                    constant: m.MAX_DRAW_BUFFERS
                }, {
                    constant: m.MAX_ELEMENT_INDEX
                }, {
                    constant: m.MAX_ELEMENTS_INDICES
                }, {
                    constant: m.MAX_ELEMENTS_VERTICES
                }, {
                    constant: m.MAX_FRAGMENT_INPUT_COMPONENTS
                }, {
                    constant: m.MAX_FRAGMENT_UNIFORM_BLOCKS
                }, {
                    constant: m.MAX_FRAGMENT_UNIFORM_COMPONENTS
                }, {
                    constant: m.MAX_PROGRAM_TEXEL_OFFSET
                }, {
                    constant: m.MAX_SAMPLES
                }, {
                    constant: m.MAX_SERVER_WAIT_TIMEOUT
                }, {
                    constant: m.MAX_TEXTURE_LOD_BIAS
                }, {
                    constant: m.MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS
                }, {
                    constant: m.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS
                }, {
                    constant: m.MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS
                }, {
                    constant: m.MAX_UNIFORM_BLOCK_SIZE
                }, {
                    constant: m.MAX_UNIFORM_BUFFER_BINDINGS
                }, {
                    constant: m.MAX_VARYING_COMPONENTS
                }, {
                    constant: m.MAX_VERTEX_OUTPUT_COMPONENTS
                }, {
                    constant: m.MAX_VERTEX_UNIFORM_BLOCKS
                }, {
                    constant: m.MAX_VERTEX_UNIFORM_COMPONENTS
                }, {
                    constant: m.MIN_PROGRAM_TEXEL_OFFSET
                }]
            }, t
        }(be),
        zt = function () {
            function e(e) {
                this.options = e, this.commandId = 0, this.context = e.context, this.version = e.version, this.onMaxCommand = new i, this.capturing = !1, this.globalCapturing = !0, this.contextInformation = {
                    context: this.context,
                    contextVersion: this.version,
                    toggleCapture: this.toggleGlobalCapturing.bind(this),
                    tagWebGlObject: this.tagWebGlObject.bind(this),
                    extensions: {}
                }, this.commandSpies = {}, this.stateSpy = new It(this.contextInformation), this.recorderSpy = new Wt(this.contextInformation), this.webGlObjectSpy = new Vt(this.contextInformation), this.analyser = new d(this.contextInformation), this.initStaticCapture(), e.recordAlways && this.spy()
            }
            return e.prototype.spy = function () {
                this.spyContext(this.context);
                var e = this.contextInformation.extensions;
                for (var t in e) e.hasOwnProperty(t) && this.spyContext(e[t])
            }, e.prototype.unSpy = function () {
                for (var e in this.commandSpies) this.commandSpies.hasOwnProperty(e) && this.commandSpies[e].unSpy()
            }, e.prototype.startCapture = function (e, t, n) {
                void 0 === e && (e = 0), void 0 === t && (t = !1), void 0 === n && (n = !1);
                var r = s.now;
                this.maxCommands = e, this.options.recordAlways || this.spy(), this.capturing = !0, this.commandId = 0, this.currentCapture = {
                    canvas: this.canvasCapture,
                    context: this.contextCapture,
                    commands: [],
                    initState: {},
                    endState: {},
                    startTime: r,
                    listenCommandsStartTime: 0,
                    listenCommandsEndTime: 0,
                    endTime: 0,
                    analyses: [],
                    frameMemory: {},
                    memory: {}
                }, this.currentCapture.canvas.width = this.context.canvas.width, this.currentCapture.canvas.height = this.context.canvas.height, this.currentCapture.canvas.clientWidth = this.context.canvas.clientWidth || this.context.canvas.width, this.currentCapture.canvas.clientHeight = this.context.canvas.clientHeight || this.context.canvas.height, this.stateSpy.startCapture(this.currentCapture, t, n), this.recorderSpy.startCapture(), this.currentCapture.listenCommandsStartTime = s.now
            }, e.prototype.stopCapture = function () {
                var e = s.now;
                return this.options.recordAlways || this.unSpy(), this.capturing = !1, this.stateSpy.stopCapture(this.currentCapture), this.recorderSpy.stopCapture(), this.currentCapture.listenCommandsEndTime = e, this.currentCapture.endTime = s.now, this.recorderSpy.appendRecordedInformation(this.currentCapture), this.analyser.appendAnalyses(this.currentCapture), this.currentCapture
            }, e.prototype.isCapturing = function () {
                return this.globalCapturing && this.capturing
            }, e.prototype.setMarker = function (e) {
                this.marker = e
            }, e.prototype.clearMarker = function () {
                this.marker = null
            }, e.prototype.log = function (e) {
                this.currentCapture.commands.push({
                    name: "LOG",
                    text: e,
                    commandArguments: [],
                    commandEndTime: s.now,
                    endTime: s.now,
                    stackTrace: [],
                    marker: "",
                    status: 40,
                    startTime: s.now,
                    result: void 0,
                    id: this.getNextCommandCaptureId()
                })
            }, e.prototype.getNextCommandCaptureId = function () {
                return this.commandId++
            }, e.prototype.onCommand = function (e, t) {
                if (this.globalCapturing && (this.webGlObjectSpy.tagWebGlObjects(t), this.recorderSpy.recordCommand(t), this.isCapturing())) {
                    var n = e.createCapture(t, this.getNextCommandCaptureId(), this.marker);
                    this.stateSpy.captureState(n), this.currentCapture.commands.push(n), n.endTime = s.now, this.maxCommands > 0 && this.currentCapture.commands.length === this.maxCommands && this.onMaxCommand.trigger(this)
                }
            }, e.prototype.spyContext = function (t) {
                var n = [];
                for (var r in t) r && n.push(r);
                for (var a = 0; a < n.length; a++)
                    if (r = n[a], !~e.unSpyableMembers.indexOf(r)) try {
                        "number" != typeof t[r] && this.spyFunction(r, t)
                    } catch (e) {
                        o.error("Cant Spy member: " + r), o.error(e)
                    }
            }, e.prototype.initStaticCapture = function () {
                var e = new jt(this.contextInformation),
                    t = e.getExtensions();
                for (var n in t) t.hasOwnProperty(n) && (this.contextInformation.extensions[n] = t[n]);
                var r = new Zt(this.contextInformation),
                    a = new Kt(this.contextInformation);
                this.contextCapture = {
                    version: this.version,
                    contextAttributes: this.context.getContextAttributes(),
                    capabilities: r.getStateData(),
                    extensions: e.getStateData(),
                    compressedTextures: a.getStateData()
                }, this.canvasCapture = {
                    width: this.context.canvas.width,
                    height: this.context.canvas.height,
                    clientWidth: this.context.canvas.clientWidth || this.context.canvas.width,
                    clientHeight: this.context.canvas.clientHeight || this.context.canvas.height,
                    browserAgent: navigator ? navigator.userAgent : ""
                }
            }, e.prototype.spyFunction = function (e, t) {
                if (0 !== e.indexOf("__SPECTOR_Origin_")) {
                    if (!this.commandSpies[e]) {
                        var n = function (e, t) {
                            var n = {};
                            for (var r in e) e.hasOwnProperty(r) && (n[r] = e[r]);
                            for (var r in t) n.hasOwnProperty(r) || (n[r] = t[r]);
                            return n
                        }(this.contextInformation, {
                            spiedCommandName: e,
                            spiedCommandRunningContext: t,
                            callback: this.onCommand.bind(this)
                        });
                        this.commandSpies[e] = new Pe(n)
                    }
                    this.commandSpies[e].spy()
                }
            }, e.prototype.toggleGlobalCapturing = function (e) {
                this.globalCapturing = e
            }, e.prototype.tagWebGlObject = function (e) {
                return this.webGlObjectSpy.tagWebGlObject(e)
            }, e.unSpyableMembers = ["canvas", "drawingBufferWidth", "drawingBufferHeight", "glp"], e
        }(),
        qt = function () {
            function e(t) {
                this.spiedWindow = t || window, this.lastFrame = 0, this.speedRatio = 1, this.willPlayNextFrame = !1, this.onFrameStart = new i, this.onFrameEnd = new i, this.onError = new i, this.lastSixtyFramesDuration = [], this.lastSixtyFramesCurrentIndex = 0, this.lastSixtyFramesPreviousStart = 0;
                for (var n = 0; n < e.fpsWindowSize; n++) this.lastSixtyFramesDuration[n] = 0;
                this.init()
            }
            return e.prototype.playNextFrame = function () {
                this.willPlayNextFrame = !0
            }, e.prototype.changeSpeedRatio = function (e) {
                this.speedRatio = e
            }, e.prototype.getFps = function () {
                for (var t = 0, n = 0; n < e.fpsWindowSize; n++) t += this.lastSixtyFramesDuration[n];
                return 0 === t ? 0 : 6e4 / t
            }, e.prototype.init = function () {
                for (var t = this, n = 0, r = e.requestAnimationFrameFunctions; n < r.length; n++) {
                    var a = r[n];
                    this.spyRequestAnimationFrame(a, this.spiedWindow)
                }
                for (var o = 0, i = e.setTimerFunctions; o < i.length; o++) a = i[o], this.spySetTimer(a);
                this.spiedWindow.VRDisplay && this.spiedWindow.addEventListener("vrdisplaypresentchange", (function (e) {
                    t.spyRequestAnimationFrame("requestAnimationFrame", e.display)
                }))
            }, e.prototype.spyRequestAnimationFrame = function (e, t) {
                var n = this;
                h.storeOriginFunction(t, e), t[e] = function () {
                    var r = arguments[0],
                        a = n.getCallback(n, r, (function () {
                            n.spiedWindow[e](r)
                        })),
                        o = h.executeOriginFunction(t, e, [a]);
                    return o
                }
            }, e.prototype.spySetTimer = function (t) {
                var n = this,
                    r = this.spiedWindow,
                    a = "setTimeout" === t;
                h.storeOriginFunction(r, t), r[t] = function () {
                    var o = arguments[0],
                        i = arguments[1],
                        s = Array.prototype.slice.call(arguments);
                    e.setTimerCommonValues.indexOf(i) > -1 && (s[0] = n.getCallback(n, o, a ? function () {
                        r[t](o)
                    } : null));
                    var u = h.executeOriginFunction(r, t, s);
                    return u
                }
            }, e.prototype.getCallback = function (t, n, r) {
                return void 0 === r && (r = null),
                    function () {
                        var a = s.now;
                        if (t.lastFrame = ++t.lastFrame % t.speedRatio, t.willPlayNextFrame || t.speedRatio && !t.lastFrame) {
                            t.onFrameStart.trigger(t);
                            try {
                                n.apply(t.spiedWindow, arguments)
                            } catch (e) {
                                t.onError.trigger(e)
                            }
                            t.lastSixtyFramesCurrentIndex = (t.lastSixtyFramesCurrentIndex + 1) % e.fpsWindowSize, t.lastSixtyFramesDuration[t.lastSixtyFramesCurrentIndex] = a - t.lastSixtyFramesPreviousStart, t.onFrameEnd.trigger(t), t.willPlayNextFrame = !1
                        } else r && r();
                        t.lastSixtyFramesPreviousStart = a
                    }
            }, e.requestAnimationFrameFunctions = ["requestAnimationFrame", "msRequestAnimationFrame", "webkitRequestAnimationFrame", "mozRequestAnimationFrame", "oRequestAnimationFrame"], e.setTimerFunctions = ["setTimeout", "setInterval"], e.setTimerCommonValues = [0, 15, 16, 33, 32, 40], e.fpsWindowSize = 60, e
        }(),
        Qt = function () {
            function e(e) {
                this.canvas = e, this.onContextRequested = new i, this.init()
            }
            return e.prototype.init = function () {
                var e = this,
                    t = function () {
                        var t = this.constructor,
                            n = e.canvas ? h.executeOriginFunction(this, "getContext", arguments) : h.executePrototypeOriginFunction(this, t, "getContext", arguments);
                        if (arguments.length > 0 && "2d" === arguments[0]) return n;
                        if (n) {
                            var r = Array.prototype.slice.call(arguments),
                                a = "webgl2" === r[0] || "experimental-webgl2" === r[0],
                                o = a ? 2 : 1;
                            e.onContextRequested.trigger({
                                context: n,
                                contextVersion: o
                            })
                        }
                        return n
                    };
                this.canvas ? (h.storeOriginFunction(this.canvas, "getContext"), this.canvas.getContext = t) : (h.storePrototypeOriginFunction(HTMLCanvasElement, "getContext"), HTMLCanvasElement.prototype.getContext = t, "undefined" != typeof OffscreenCanvas && (h.storePrototypeOriginFunction(OffscreenCanvas, "getContext"), OffscreenCanvas.prototype.getContext = t))
            }, e
        }(),
        Jt = function () {
            function e() {
                this.noFrameTimeout = -1, this.captureNextFrames = 0, this.captureNextCommands = 0, this.quickCapture = !1, this.fullCapture = !1, this.retry = 0, this.contexts = [], this.timeSpy = new qt, this.onCaptureStarted = new i, this.onCapture = new i, this.onError = new i, this.timeSpy.onFrameStart.add(this.onFrameStart, this), this.timeSpy.onFrameEnd.add(this.onFrameEnd, this), this.timeSpy.onError.add(this.onErrorInternal, this)
            }
            return e.getFirstAvailable3dContext = function (e) {
                return this.tryGetContextFromHelperField(e) || this.tryGetContextFromCanvas(e, "webgl") || this.tryGetContextFromCanvas(e, "experimental-webgl") || this.tryGetContextFromCanvas(e, "webgl2") || this.tryGetContextFromCanvas(e, "experimental-webgl2")
            }, e.tryGetContextFromHelperField = function (e) {
                var t = e instanceof HTMLCanvasElement ? e.getAttribute("__spector_context_type") : e.__spector_context_type;
                if (t) return this.tryGetContextFromCanvas(e, t)
            }, e.tryGetContextFromCanvas = function (e, t) {
                var n;
                try {
                    n = e.getContext(t)
                } catch (e) {}
                return n
            }, e.prototype.rebuildProgramFromProgramId = function (e, t, n, r, a) {
                var o = Tt.getFromGlobalStore(e);
                this.rebuildProgram(o, t, n, r, a)
            }, e.prototype.rebuildProgram = function (e, t, n, a, o) {
                r.rebuildProgram(e, t, n, a, o)
            }, e.prototype.referenceNewProgram = function (e, t) {
                Tt.updateInGlobalStore(e, t)
            }, e.prototype.pause = function () {
                this.timeSpy.changeSpeedRatio(0)
            }, e.prototype.play = function () {
                this.timeSpy.changeSpeedRatio(1)
            }, e.prototype.playNextFrame = function () {
                this.timeSpy.playNextFrame()
            }, e.prototype.drawOnlyEveryXFrame = function (e) {
                this.timeSpy.changeSpeedRatio(e)
            }, e.prototype.getFps = function () {
                return this.timeSpy.getFps()
            }, e.prototype.spyCanvases = function () {
                this.canvasSpy ? this.onErrorInternal("Already spying canvas.") : (this.canvasSpy = new Qt, this.canvasSpy.onContextRequested.add(this.spyContext, this))
            }, e.prototype.spyCanvas = function (e) {
                this.canvasSpy ? this.onErrorInternal("Already spying canvas.") : (this.canvasSpy = new Qt(e), this.canvasSpy.onContextRequested.add(this.spyContext, this))
            }, e.prototype.getAvailableContexts = function () {
                return this.getAvailableContexts()
            }, e.prototype.captureCanvas = function (t, n, r, a) {
                void 0 === n && (n = 0), void 0 === r && (r = !1), void 0 === a && (a = !1);
                var i = this.getAvailableContextSpyByCanvas(t);
                if (i) this.captureContextSpy(i, n, r, a);
                else {
                    var s = e.getFirstAvailable3dContext(t);
                    s ? this.captureContext(s, n, r, a) : o.error("No webgl context available on the chosen canvas.")
                }
            }, e.prototype.captureContext = function (e, t, n, r) {
                void 0 === t && (t = 0), void 0 === n && (n = !1), void 0 === r && (r = !1);
                var a = this.getAvailableContextSpyByCanvas(e.canvas);
                a || ((a = e.getIndexedParameter ? new zt({
                    context: e,
                    version: 2,
                    recordAlways: !1
                }) : new zt({
                    context: e,
                    version: 1,
                    recordAlways: !1
                })).onMaxCommand.add(this.stopCapture, this), this.contexts.push({
                    canvas: a.context.canvas,
                    contextSpy: a
                })), a && this.captureContextSpy(a, t, n, r)
            }, e.prototype.captureContextSpy = function (e, t, n, r) {
                var a = this;
                void 0 === t && (t = 0), void 0 === n && (n = !1), void 0 === r && (r = !1), this.quickCapture = n, this.fullCapture = r, this.capturingContext ? this.onErrorInternal("Already capturing a context.") : (this.retry = 0, this.capturingContext = e, this.capturingContext.setMarker(this.marker), (t = Math.min(t, 1e5)) > 0 ? this.captureCommands(t) : this.captureFrames(1), this.noFrameTimeout = setTimeout((function () {
                    t > 0 ? a.stopCapture() : a.capturingContext && a.retry > 1 ? a.onErrorInternal("No frames with gl commands detected. Try moving the camera.") : a.onErrorInternal("No frames detected. Try moving the camera or implementing requestAnimationFrame.")
                }), 1e4))
            }, e.prototype.captureNextFrame = function (e, t, n) {
                void 0 === t && (t = !1), void 0 === n && (n = !1), e instanceof HTMLCanvasElement || self.OffscreenCanvas && e instanceof OffscreenCanvas ? this.captureCanvas(e, 0, t, n) : this.captureContext(e, 0, t, n)
            }, e.prototype.startCapture = function (e, t, n, r) {
                void 0 === n && (n = !1), void 0 === r && (r = !1), e instanceof HTMLCanvasElement || self.OffscreenCanvas && e instanceof OffscreenCanvas ? this.captureCanvas(e, t, n, r) : this.captureContext(e, t, n, r)
            }, e.prototype.stopCapture = function () {
                if (this.capturingContext) {
                    var e = this.capturingContext.stopCapture();
                    if (e.commands.length > 0) return this.noFrameTimeout > -1 && clearTimeout(this.noFrameTimeout), this.triggerCapture(e), this.capturingContext = void 0, this.captureNextFrames = 0, this.captureNextCommands = 0, e;
                    0 === this.captureNextCommands && (this.retry++, this.captureFrames(1))
                }
            }, e.prototype.setMarker = function (e) {
                this.marker = e, this.capturingContext && this.capturingContext.setMarker(e)
            }, e.prototype.clearMarker = function () {
                this.marker = null, this.capturingContext && this.capturingContext.clearMarker()
            }, e.prototype.log = function (e) {
                this.capturingContext && this.capturingContext.log(e)
            }, e.prototype.captureFrames = function (e) {
                this.captureNextFrames = e, this.captureNextCommands = 0, this.playNextFrame()
            }, e.prototype.captureCommands = function (e) {
                this.captureNextFrames = 0, this.captureNextCommands = e, this.play(), this.capturingContext ? (this.onCaptureStarted.trigger(void 0), this.capturingContext.startCapture(e, this.quickCapture, this.fullCapture)) : (this.onErrorInternal("No context to capture from."), this.captureNextCommands = 0)
            }, e.prototype.spyContext = function (e) {
                var t = this.getAvailableContextSpyByCanvas(e.context.canvas);
                t || ((t = new zt({
                    context: e.context,
                    version: e.contextVersion,
                    recordAlways: !0
                })).onMaxCommand.add(this.stopCapture, this), this.contexts.push({
                    canvas: t.context.canvas,
                    contextSpy: t
                })), t.spy()
            }, e.prototype.getAvailableContextSpyByCanvas = function (e) {
                for (var t = 0, n = this.contexts; t < n.length; t++) {
                    var r = n[t];
                    if (r.canvas === e) return r.contextSpy
                }
            }, e.prototype.onFrameStart = function () {
                this.captureNextCommands > 0 || (this.captureNextFrames > 0 ? (this.capturingContext && (this.onCaptureStarted.trigger(void 0), this.capturingContext.startCapture(0, this.quickCapture, this.fullCapture)), this.captureNextFrames--) : this.capturingContext = void 0)
            }, e.prototype.onFrameEnd = function () {
                this.captureNextCommands > 0 || 0 === this.captureNextFrames && this.stopCapture()
            }, e.prototype.triggerCapture = function (e) {
                this.onCapture.trigger(e)
            }, e.prototype.onErrorInternal = function (e) {
                if (o.error(e), this.noFrameTimeout > -1 && clearTimeout(this.noFrameTimeout), !this.capturingContext) throw e;
                this.capturingContext = void 0, this.captureNextFrames = 0, this.captureNextCommands = 0, this.retry = 0, this.onError.trigger(e)
            }, e
        }();
    return t
})()));

let spector;
let displayUIFlag = false;
let captureCanvas;

export const initSpector = (port, canvas) => {
	if (!spector) {
		spector = new SPECTOR.Spector();
        spector.onCapture.add((capture) => {
			// 收到捕获数据，如果当前没有显示ui，则将数据上传到服务器
			if (!displayUIFlag) {
				save(port || 0, "webgl_cmd", `cmd_${Date.now()}.gl_cmd`, JSON.stringify(capture));
				// console.log("Captured", capture);
			}
        });
		// spyCanvas， 重载了getContext方法， getContext被调用后， 会发出通知（通知函数重载该gl上下文中的其他指令方法）
		captureCanvas = canvas || window["canvas"];
		spector.spyCanvas(captureCanvas);

		setTimeout(() => {
			recordWebgl();
		}, 5000);
	}
}


export const displayUI = () => {
	if (!spector) {
		initSpector();
	} else if (displayUIFlag) {
		return;
	}
	spector.displayUI();
	displayUIFlag = true;
}

export const recordWebgl = (commandCount = 0, quickCapture = false, fullCapture = false) => {
	initSpector();
    // spector.displayUI();
    spector.captureCanvas(captureCanvas, commandCount, quickCapture, fullCapture);
}

let uploadPort;
export const setPort = (p) => {
	uploadPort = p;
}

/**
 * 保存文件到项目目录下
 * @param path
 * @param content
 * @param success
 * @param fail
 */
const save = (port, dir, filename, content, success, fail) => {
	console.log("save len: ", content.length);
	// let form: FormData | undefined = new FormData();
	const sPath = `${dir?dir + "/":""}${filename}`;
	// form.append("content", new Blob([content]), sPath);
	// let hostname = inspectedWindowUrl.split("://")[1].split("/")[0];
	var xhr = new XMLHttpRequest();
	xhr.open('POST', `http://${location.hostname}:${port || uploadPort || 90}/upload`);
	xhr.setRequestHeader("Content-Type", "application/json");
	xhr.addEventListener('error', r => console.error("", r), false);
	xhr.addEventListener('readystatechange', _r => {
		if (xhr.readyState === 4 && xhr.status === 200) {
			if (xhr.response !== '' || xhr.response !== undefined || xhr.response !== null) {
				success && success(xhr.response);
				console.log(`save ${filename} success`, xhr.response);
			}
			else {
				fail && fail(xhr.response);
				console.error(`save ${filename} result :\n` + xhr.response);
				alert(`怎么搞的😒😒😒,输个路径都输不对🙈\n${dir}`);
			}
		}
	}, false);
	let r = {filename: sPath, content: content};
	// let r = `\n------WebKitFormBoundary4dHtQIk39pfgJ6i6\nContent-Disposition: form-data; name="content"; filename="${sPath}"\r\n${content}\n------WebKitFormBoundary4dHtQIk39pfgJ6i6--`;
	xhr.send(JSON.stringify(r));
	console.log( r);
	// form = undefined;
}

