class Diedeadcode < Formula
  desc "Conservative TypeScript dead code detection"
  homepage "https://github.com/dean0x/diedeadcode"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/dean0x/diedeadcode/releases/download/v0.1.0/ddd-darwin-arm64"
      sha256 "68cb8b571adc0e668139fa13cac03ed12af62d63ca7c42d1264bf14b5722c224"
    else
      url "https://github.com/dean0x/diedeadcode/releases/download/v0.1.0/ddd-darwin-x64"
      sha256 "fb96417dac5dc9e268c82404e77bb5995f4a81d50230b34740ea24b36fdc565f"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/dean0x/diedeadcode/releases/download/v0.1.0/ddd-linux-arm64"
      sha256 "8ab01135c0dbc64fa9151962b789edec1eb7678a19d1e2761673971335409daf"
    else
      url "https://github.com/dean0x/diedeadcode/releases/download/v0.1.0/ddd-linux-x64"
      sha256 "b6e32f42926b84ce16d54981ec1ca93733160264f819af320d39b4cb6bf7408c"
    end
  end

  def install
    binary_name = "ddd-#{OS.kernel_name.downcase}-#{Hardware::CPU.arch == :arm64 ? "arm64" : "x64"}"
    bin.install binary_name => "ddd"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/ddd --version")
  end
end
