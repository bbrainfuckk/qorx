class Qorx < Formula
  desc "Community Edition CLI for local context-resolution research"
  homepage "https://github.com/bbrainfuckk/qorx"
  url "https://github.com/bbrainfuckk/qorx.git",
      tag:      "v0.1-ylem"
  version "0.1.0-ylem"
  license "AGPL-3.0-only"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match "qorx 0.1.0-ylem", shell_output("#{bin}/qorx --version")
  end
end
