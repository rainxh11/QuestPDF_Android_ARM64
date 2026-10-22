using System;
using System.Linq;
using System.Runtime.InteropServices;

namespace QuestPDF.Helpers;

internal static class NativeRuntimeDetection
{
    internal static bool IsAndroidRuntime
    {
        get
        {
#if NET5_0_OR_GREATER
            if (OperatingSystem.IsAndroid() || RuntimeInformation.RuntimeIdentifier.StartsWith("linux-bionic-", StringComparison.Ordinal))
                return true;
#endif
            return RuntimeInformation.IsOSPlatform(OSPlatform.Create("ANDROID"));
        }
    }

    public static readonly string[] SupportedPlatforms =
    {
        "win-x86",
        "win-x64",
        "win-arm64",
        "linux-x64",
        "linux-musl-x64",
        "linux-arm64",
        "osx-x64",
        "osx-arm64",
        "android-arm64",
        "linux-bionic-arm64",
        "android-x64",
        "linux-bionic-x64"
    };

    public static bool IsCurrentRuntimeSupported()
    {
        return SupportedPlatforms.Contains(RuntimePlatform.Value);
    }

    public static readonly Lazy<string> RuntimePlatform = new(() =>
    {
        return $"{GetSystemIdentifier()}-{GetProcessArchitecture()}";

        static string GetSystemIdentifier()
        {
#if NET5_0_OR_GREATER
            // Standalone NativeAOT uses a Bionic RID rather than the Android workload RID.
            if (RuntimeInformation.RuntimeIdentifier.StartsWith("linux-bionic-", StringComparison.Ordinal))
                return "linux-bionic";

            if (OperatingSystem.IsAndroid())
                return "android";
#endif
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Create("ANDROID")))
                return "android";

            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
                return "win";

            if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
                return IsGlibcLinux.Value ? "linux" : "linux-musl";

            if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
                return "osx";

            return "other";
        }

        static string GetProcessArchitecture()
        {
            return RuntimeInformation.ProcessArchitecture.ToString().ToLower();
        }
    });

    private static readonly Lazy<string?> GlibcVersionText = new(() =>
    {
        if (!RuntimeInformation.IsOSPlatform(OSPlatform.Linux) || IsAndroidRuntime)
            return null;

        try
        {
            return Marshal.PtrToStringAnsi(gnu_get_libc_version());
        }
        catch
        {
            return null;
        }
    });

    private static readonly Lazy<bool> IsGlibcLinux = new(() =>
        RuntimeInformation.IsOSPlatform(OSPlatform.Linux) && GlibcVersionText.Value != null);

    public static readonly Lazy<Version?> GlibcVersion = new(() =>
        Version.TryParse(GlibcVersionText.Value, out var version) ? version : null);

    [DllImport("libc", EntryPoint = "gnu_get_libc_version")]
    private static extern IntPtr gnu_get_libc_version();
}
