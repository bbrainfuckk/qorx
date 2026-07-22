Name:           qorx
Version:        0.0.1_ylem
Release:        1%{?dist}
Summary:        Qorx AI-native language and runtime for local context resolution
License:        AGPL-3.0-only
URL:            https://github.com/bbrainfuckk/qorx
Source0:        https://github.com/bbrainfuckk/qorx/archive/refs/tags/v%{version}.tar.gz
BuildRequires:  cargo
BuildRequires:  rust

%description
Qorx is an AI-native programming language and local runtime for context resolution.

%prep
%autosetup -n qorx-%{version}

%build
cargo build --release --locked

%check
cargo test --locked

%install
install -Dm755 target/release/qorx %{buildroot}%{_bindir}/qorx
install -Dm644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -Dm644 docs/COMMANDS.md %{buildroot}%{_docdir}/%{name}/COMMANDS.md
install -Dm644 LICENSE %{buildroot}%{_licensedir}/%{name}/LICENSE

%files
%{_bindir}/qorx
%{_docdir}/%{name}/README.md
%{_docdir}/%{name}/COMMANDS.md
%license %{_licensedir}/%{name}/LICENSE

%changelog
* Wed May 06 2026 Marvin Sarreal Villanueva <marvin@orin.work> - 0.0.1_ylem-1
- Roll Qorx packaging to the 0.0.1-ylem line.

* Sat May 02 2026 Marvin Sarreal Villanueva <marvin@orin.work> - 1.0.3-1
- Package Qorx distribution release.
