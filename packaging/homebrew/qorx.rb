class Qorx < Formula
  desc "AI-native language and runtime for local context resolution"
  homepage "https://github.com/bbrainfuckk/qorx"
  url "https://github.com/bbrainfuckk/qorx.git",
      tag:      "v0.0.1-ylem"
  license "AGPL-3.0-only"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--path", ".", "--root", prefix
  end

  test do
    assert_match "qorx 0.0.1-ylem", shell_output("#{bin}/qorx --version")
  end
end
