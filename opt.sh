iwasi=./rs_jptxt2tokens4web.wasm

wasm-opt \
	-Oz \
	-o opt.wasm \
	--enable-bulk-memory \
	--enable-nontrapping-float-to-int \
	--enable-simd \
	"${iwasi}"
