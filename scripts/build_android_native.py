"""Build actual QuestPDF native forks for Android ARMv7, ARM64 or x86-64."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import runpy
from concurrent.futures import ThreadPoolExecutor

ROOT = Path(__file__).resolve().parents[1]
LOCK = json.loads((ROOT / 'native/android/sources.json').read_text())


def run(*args, cwd=ROOT, env=None):
    print('+', ' '.join(str(a) for a in args), flush=True)
    subprocess.run([str(a) for a in args], cwd=cwd, env=env, check=True)


def copy_changed_source(source, destination):
    # Preserve unchanged files, but timestamp changed content now. copy2 alone
    # can restore an older upstream timestamp and leave stale Ninja objects.
    destination = Path(destination)
    if not destination.exists() or Path(source).read_bytes() != destination.read_bytes():
        shutil.copy2(source, destination)
        os.utime(destination, None)
    return str(destination)


def source(name):
    dest = ROOT / 'artifacts/upstream' / name
    spec = LOCK[name]
    if not dest.exists():
        run('git', 'init', dest)
        run('git', 'remote', 'add', 'origin', spec['url'], cwd=dest)
        if name == 'skia':
            run('git', 'config', 'remote.origin.promisor', 'true', cwd=dest)
            run('git', 'config', 'remote.origin.partialclonefilter', 'blob:none', cwd=dest)
            run('git', 'sparse-checkout', 'init', '--no-cone', cwd=dest)
            (dest / '.git/info/sparse-checkout').write_text(
                '/*\n!/resources/\n!/tests/\n!/docs/\n!/site/\n!/gm/\n!/bench/\n!/infra/\n!/example/\n!/demos/\n')
    if not (dest / '.git').is_dir():
        raise RuntimeError(f'{dest} exists without its own Git metadata; restore it before building')
    head = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=dest, capture_output=True, text=True)
    if head.stdout.strip() != spec['commit']:
        filters = ['--filter=blob:none'] if name == 'skia' else []
        run('git', '-c', 'core.longpaths=true', 'fetch', '--depth=1', *filters, 'origin', spec['commit'], cwd=dest)
        run('git', '-c', 'core.longpaths=true', 'checkout', '--detach', spec['commit'], cwd=dest)
    return dest


def sync_skia(skia):
    # Read Skia's pinned dependency revisions. GPU, tools, ICU and test-data repos
    # are unnecessary for this CPU/PDF build and would add many gigabytes.
    namespace = runpy.run_path(str(skia / 'tools/git-sync-deps'))
    deps = namespace['parse_file_to_dict'](str(skia / 'DEPS'))['deps']
    # This QuestPDF fork already restricts DEPS to the CPU/PDF build. ICU is
    # still needed for its small bidi subset even with skia_use_icu=false.
    names = [key.removeprefix('third_party/externals/') for key, value in deps.items()
             if key.startswith('third_party/externals/') and isinstance(value, str)]

    def fetch(name):
        relative = 'third_party/externals/' + name
        url, commit = deps[relative].rsplit('@', 1)
        dest = skia / relative
        if not (dest / '.git').exists():
            run('git', 'init', dest)
            run('git', 'remote', 'add', 'origin', url, cwd=dest)
        head = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=dest, capture_output=True, text=True)
        if head.stdout.strip() != commit:
            run('git', 'fetch', '--depth=1', 'origin', commit, cwd=dest)
            run('git', '-c', 'core.longpaths=true', 'checkout', '--detach', commit, cwd=dest)

    with ThreadPoolExecutor(max_workers=4) as executor:
        list(executor.map(fetch, names))
    if not (skia / ('bin/gn.exe' if os.name == 'nt' else 'bin/gn')).exists():
        run(sys.executable, 'bin/fetch-gn', cwd=skia)


def generate_grapheme_headers(skia, host_cc):
    # Skia's host-toolchain test does not generate these tables when Android is
    # cross-compiled on Windows. Generate the pinned Unicode tables with a host
    # compiler before invoking Ninja. The same tables are shared by all ABIs.
    grapheme = skia / 'third_party/externals/libgrapheme'
    tools = ROOT / 'artifacts/build/grapheme-host-tools'
    tools.mkdir(parents=True, exist_ok=True)
    for name in ['bidirectional', 'case', 'character', 'line', 'sentence', 'word']:
        header = grapheme / f'gen/{name}.h'
        if header.exists() and header.stat().st_size > 0:
            continue
        exe = tools / (name + ('.exe' if os.name == 'nt' else ''))
        run(host_cc, '-std=c99', '-O2', grapheme / f'gen/{name}.c',
            grapheme / 'gen/util.c', '-o', exe)
        temporary = header.with_suffix('.h.tmp')
        with temporary.open('wb') as output:
            subprocess.run([str(exe)], cwd=grapheme, stdout=output, check=True)
        temporary.replace(header)


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument('--ndk', required=True, type=Path)
    parser.add_argument('--cmake', default='cmake')
    parser.add_argument('--ninja', default='ninja')
    parser.add_argument('--jobs', type=int, default=2)
    parser.add_argument('--only', choices=['skia', 'qpdf', 'all'], default='all')
    parser.add_argument('--skip-sync', action='store_true')
    parser.add_argument('--abi', choices=['armeabi-v7a', 'arm64-v8a', 'x86_64'], default='arm64-v8a')
    parser.add_argument('--host-cc', default='gcc' if os.name == 'nt' else 'cc')
    args = parser.parse_args()
    ndk = args.ndk.resolve()
    cpu = {'armeabi-v7a': 'arm', 'arm64-v8a': 'arm64', 'x86_64': 'x64'}[args.abi]
    output = ROOT / 'artifacts/android' / args.abi
    output.mkdir(parents=True, exist_ok=True)
    cmake_args = ['-G', 'Ninja', f'-DCMAKE_MAKE_PROGRAM={args.ninja}',
                  f'-DCMAKE_TOOLCHAIN_FILE={ndk.as_posix()}/build/cmake/android.toolchain.cmake',
                  f'-DANDROID_ABI={args.abi}', '-DANDROID_PLATFORM=android-24',
                  '-DANDROID_STL=c++_static', '-DCMAKE_BUILD_TYPE=Release']
    if args.only in ('qpdf', 'all'):
        qpdf = source('qpdf')
        build = ROOT / f'artifacts/build/android-qpdf-{cpu}'
        run(args.cmake, '-S', ROOT / 'native/android', '-B', build, *cmake_args,
            '-DQUESTPDF_BUILD_QPDF=ON', f'-DQPDF_DIR={qpdf.as_posix()}')
        run(args.cmake, '--build', build, '--target', 'libqpdf', '--parallel', args.jobs)
        shutil.copy2(build / 'qpdf/libqpdf/libquestpdf_qpdf.so', output)
    if args.only in ('skia', 'all'):
        skia = source('skia')
        wrapper = source('skia-build')
        if not args.skip_sync:
            sync_skia(skia)
        generate_grapheme_headers(skia, args.host_cc)
        gn = skia / ('bin/gn.exe' if os.name == 'nt' else 'bin/gn')
        gn_args = '''is_official_build=true is_component_build=false is_debug=false
            target_os="android" ndk_api=24
            skia_enable_tools=false skia_enable_ganesh=false
            skia_use_gl=false skia_use_vulkan=false skia_use_dng_sdk=false
            skia_use_icu=false skia_use_icu4x=false skia_use_libgrapheme=true
            skia_use_harfbuzz=true skia_use_fontconfig=false skia_use_freetype=true
            skia_enable_fontmgr_android=true skia_enable_fontmgr_custom_directory=true
            skia_use_system_expat=false skia_use_system_icu=false
            skia_use_system_harfbuzz=false skia_use_system_libjpeg_turbo=false
            skia_use_system_libpng=false skia_use_system_libwebp=false
            skia_use_system_zlib=false skia_use_system_freetype2=false
            skia_use_libjpeg_turbo_decode=true skia_use_libjpeg_turbo_encode=true
            skia_use_libpng_encode=true skia_use_libpng_decode=true
            skia_use_libwebp_encode=true skia_use_libwebp_decode=true
            skia_enable_svg=true skia_use_expat=true skia_enable_skshaper=true
            skia_enable_skunicode=true skia_pdf_subset_harfbuzz=true skia_enable_pdf=true
            skia_enable_android_utils=false skia_compile_modules=false
            skia_use_jpeg_gainmaps=false skia_use_lua=false
            skia_use_libavif=false skia_use_libjxl_decode=false
            skia_use_partition_alloc=false
            extra_cflags=["-fPIC","-fno-rtti","-ffunction-sections","-fdata-sections"]'''
        gn_args += f' ndk="{ndk.as_posix()}" target_cpu="{cpu}"'
        skia_output = skia / f'out/android-{cpu}'
        run(gn, 'gen', f'out/android-{cpu}', '--args=' + ' '.join(gn_args.split()), cwd=skia)
        run(args.ninja, '-C', skia_output, '-j', args.jobs,
            'skia', 'svg', 'skparagraph', 'skresources')
        # Copy the wrapper before the Android-only adaptation; keep upstream intact.
        patched = ROOT / f'artifacts/build/android-wrapper-{cpu}'
        shutil.copytree(wrapper / 'native', patched, dirs_exist_ok=True,
                        ignore=shutil.ignore_patterns('build'), copy_function=copy_changed_source)
        fonts = patched / 'src/text/fontManager.cpp'
        text = fonts.read_text()
        old = '#ifdef __linux__'
        if old not in text:
            raise RuntimeError('Upstream font wrapper changed; review Android adaptation')
        text = text.replace(old, '''#ifdef __ANDROID__
#include "include/ports/SkFontMgr_android.h"
#include "include/ports/SkFontScanner_FreeType.h"
sk_sp<SkFontMgr> font_manager_create_default_internal() {
    return SkFontMgr_New_Android(nullptr, SkFontScanner_Make_FreeType());
}
#endif
#if defined(__linux__) && !defined(__ANDROID__)''')
        fonts.write_text(text)
        build = ROOT / f'artifacts/build/android-skia-{cpu}'
        run(args.cmake, '-S', ROOT / 'native/android', '-B', build, *cmake_args,
            '-DQUESTPDF_BUILD_SKIA=ON', f'-DSKIA_DIR={skia.as_posix()}',
            f'-DSKIA_OUTPUT_DIR={skia_output.as_posix()}',
            f'-DWRAPPER_DIR={patched.as_posix()}')
        run(args.cmake, '--build', build, '--target', 'questpdf_skia', '--parallel', args.jobs)
        shutil.copy2(build / 'libquestpdf_skia.so', output)
    host = 'windows-x86_64' if os.name == 'nt' else 'linux-x86_64'
    strip = ndk / 'toolchains/llvm/prebuilt' / host / 'bin' / ('llvm-strip.exe' if os.name == 'nt' else 'llvm-strip')
    for lib in output.glob('libquestpdf_*.so'):
        run(strip, '--strip-unneeded', lib)
    # Both RIDs deliberately refer to Bionic binaries, never the glibc ARM64 files.
    for rid in (f'android-{cpu}', f'linux-bionic-{cpu}'):
        dest = ROOT / 'src/dotnet/library/QuestPDF/Runtimes' / rid / 'native'
        dest.mkdir(parents=True, exist_ok=True)
        for lib in output.glob('libquestpdf_*.so'):
            shutil.copy2(lib, dest)
    (output / 'sources.json').write_text(json.dumps(LOCK, indent=2) + '\n')


if __name__ == '__main__':
    main()
