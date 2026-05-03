class Qorx < Formula
  desc "Community Edition CLI for local context-resolution research"
  homepage "https://github.com/bbrainfuckk/qorx"
  url "https://github.com/bbrainfuckk/qorx.git",
      tag:      "v1.0.4"
  version "1.0.4"
  license "AGPL-3.0-only"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match "qorx 1.0.4", shell_output("#{bin}/qorx --version")
  end
end
