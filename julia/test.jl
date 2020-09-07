using Test

struct Tick
  timestamp::Ptr{Vector{Int64}}
  bid::Ptr{Vector{Int64}}
  ask::Ptr{Vector{Int64}}
end

struct TS
  ticks::Vector{Tick}
end


struct Account
  cash::Ptr{Int64}
  portfolio::Ptr{Vector{Int64}}
end

struct Engine
  acct::Account
  time::UInt64
  prices::TS
  index::Int64
  indicators::Ptr{Vector{Int64}}
end

@testset "working?" begin
  @ccall "target/release/rsbacktester".test_rust(34::Int64)::Cvoid
  engine = @ccall "target/release/rsbacktester".jl_engine("test_resources/ticks.csv"::Cstring, 10000::Int64)::Ptr{Engine}
  resp = @ccall "target/release/rsbacktester".engine_json(engine::Ptr{Engine})::Ptr{UInt8}
  println(resp)
  println(unsafe_string(resp))
  #@ccall "julia/librsbacktester".step_jl_engine(engine::Ptr, 10::UInt32)::Cvoid
  #println(resp)
  #@assert resp == "a"
end
