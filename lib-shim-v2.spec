#needsrootforbuild
%global __cargo_skip_build 0
%global _debugsource_packages 1
%global _debuginfo_subpackages 1
%define _unpackaged_files_terminate_build 0
%define debug_package %{nil}

Name:           lib-shim-v2
Version:        0.0.1
Release:        3
URL:            https://gitee.com/openeuler/lib-shim-v2
Source:         %{name}-%{version}.tar.gz
Summary:        lib-shim-v2 is shim v2 ttrpc client which is called by iSulad.
Group:          Application/System
License:        Mulan PSL v2

ExclusiveArch:  x86_64 aarch64

BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  rust-packaging
BuildRequires:  gcc

%description
Based on Rust programming language, as a shim v2 ttrpc client, it is called by iSulad.

%package devel
Summary: shim v2 ttrpc client
Group:   Libraries
ExclusiveArch:  x86_64 aarch64
Requires: %{name} = %{version}-%{release}

%description devel
the %{name}-libs package contains Libraries for shim v2 ttrpc client 

%prep
%autosetup -p1
%cargo_prep
%cargo_generate_buildrequires

%build
sed -i '/\[source.crates-io\]/{n;d}' ./.cargo/config
sed -i '/\[source.local-registry\]/{n;d}' ./.cargo/config
sed -i '/\[source.local-registry\]/a directory = "vendor"' ./.cargo/config
%cargo_build -a

%install
mkdir -p ${RPM_BUILD_ROOT}/{%{_libdir},%{_includedir}}
install -m 0644 shim_v2.h ${RPM_BUILD_ROOT}/%{_includedir}/shim_v2.h
install -m 0755 target/release/libshim_v2.so ${RPM_BUILD_ROOT}/%{_libdir}/libshim_v2.so

%files
%defattr(-,root, root,-)
%{_libdir}/*

%files devel
%defattr(-,root, root,-)
%{_includedir}/shim_v2.h

%changelog
* Mon Jun 28 2021 gaohuatao <gaohuatao@huawei.com> - 0.0.1-3
- Type:NA
- ID:NA
- SUG:NA
- DESC:improve privileges

* Thu Jun 24 2021 gaohuatao <gaohuatao@huawei.com> - 0.0.1-2
- Type:NA
- ID:NA
- SUG:NA
- DESC:add Cargo.lock

* Mon Jun 21 2021 gaohuatao <gaohuatao@huawei.com> - 0.0.1
- Initial RPM release
