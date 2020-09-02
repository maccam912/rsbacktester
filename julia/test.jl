using Test

@testset "working?" begin
  @ccall "julia/librsbacktester".test_rust(34::Int64)::Cvoid
  engine = @ccall "julia/librsbacktester".jl_engine("test_resources/ticks.csv"::Cstring, 10000::Int64)::Ptr
  @ccall "julia/librsbacktester".step_jl_engine(engine::Ptr, 10::UInt32)::Cvoid
  #resp = @ccall lib.engine_json(engine::Ptr)::Cstring
  #println(resp)
  #@assert resp == "a"
end
