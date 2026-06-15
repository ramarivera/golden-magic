(module
  (memory (export "memory") 1)
  (data (i32.const 2048) "{\"protocol\":\"golden-magic.wasm-json.v1\",\"rows\":[{\"name\":\"alpha\",\"status\":\"ok\"},{\"name\":\"beta\",\"status\":\"degraded\"}]}")
  (func (export "golden_magic_parse") (param $ptr i32) (param $len i32) (result i64)
    (i64.or
      (i64.shl (i64.const 2048) (i64.const 32))
      (i64.const 116)
    )
  )
)
