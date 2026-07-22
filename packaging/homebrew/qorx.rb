class Qorx < Formula
  desc "Agnostic language and runtime for local context resolution"
  homepage "https://github.com/bbrainfuckk/qorx"
  url "https://github.com/bbrainfuckk/qorx.git",
      tag:      "v1.0.6"
  license "AGPL-3.0-only"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--path", ".", "--root", prefix
  end

  test do
    assert_match "qorx 1.0.6", shell_output("#{bin}/qorx --version")
  end
end
