using Test

@testset "working?" begin
  @test (@ccall "julia/librsbacktester".test()::Cint) == 1
end
