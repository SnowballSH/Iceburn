### A Pluto.jl notebook ###
# v0.14.7

using Markdown
using InteractiveUtils

# ╔═╡ 4d454b33-c288-4909-a20d-3d744dbf742d
using Plots

# ╔═╡ c0934e70-294e-11ec-0eef-3f0b731d8cb3
function timing(length::Float64)
	rate::Float64 = 1;
	if length >= 10.0
		if length <= 18.0
			rate *= 0.90 + (length - 10.0) * 0.45
		elseif length <= 24.0
			rate *= 0.90 + (18.0 - 10.0) * 0.45
		elseif length <= 28.0
			rate *= 0.80 + (28.0 - length) * 0.25
		elseif length <= 70.0
			rate *= 0.10 + 5.00 / (length - 20.0)
		else
			rate *= 0.03 + (150.0 - length) * 0.0002
		end
	else
		rate *= 0.15 + 4.00 / (-length + 18.0)
	end
	return rate * 0.95
end

# ╔═╡ cc4bfdda-a837-4b07-8c7a-639fb717d18b
begin
	time::Float64 = 180000.0
	x = 0:85
	y = [[], [], []]
	for t in x
		final = timing(Float64(t))
		use = max(time / max(5.0, 85.0 - Float64(t)) * final, 0.1)
		push!(y[1], final)
		push!(y[2], use / 1000)
		time -= use
		push!(y[3], time / 1000 / 60)
	end
	plot(x, y, lw=2, layout = (3, 1))
end

# ╔═╡ Cell order:
# ╠═4d454b33-c288-4909-a20d-3d744dbf742d
# ╠═c0934e70-294e-11ec-0eef-3f0b731d8cb3
# ╠═cc4bfdda-a837-4b07-8c7a-639fb717d18b
