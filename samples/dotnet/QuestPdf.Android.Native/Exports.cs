using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;

namespace QuestPdf.Android.Native;

/// <summary>Version 1 C ABI. Every non-null output buffer belongs to the caller;
/// release it with questpdf_free, including buffers returned on errors.</summary>
public static unsafe class Exports
{
    [UnmanagedCallersOnly(EntryPoint = "questpdf_abi_version", CallConvs = [typeof(CallConvCdecl)])]
    public static int AbiVersion() => 1;

    [UnmanagedCallersOnly(EntryPoint = "questpdf_generate", CallConvs = [typeof(CallConvCdecl)])]
    public static int Generate(int feature, byte* directory, byte** output, int* length)
    {
        if (output == null || length == null)
            return 2;
        *output = null;
        *length = 0;
        try
        {
            var path = Marshal.PtrToStringUTF8((IntPtr)directory)
                ?? throw new ArgumentException("A writable app cache directory is required.");
            var data = FeatureSuite.Generate(feature, path);
            ReturnBuffer(data, output, length);
            return 0;
        }
        catch (Exception exception)
        {
            ReturnBuffer(Encoding.UTF8.GetBytes(exception.ToString()), output, length);
            return 1;
        }
    }

    [UnmanagedCallersOnly(EntryPoint = "questpdf_run_tests", CallConvs = [typeof(CallConvCdecl)])]
    public static int RunTests(byte* directory, byte** output, int* length)
    {
        if (output == null || length == null)
            return 2;
        *output = null;
        *length = 0;
        try
        {
            var path = Marshal.PtrToStringUTF8((IntPtr)directory)
                ?? throw new ArgumentException("A writable app cache directory is required.");
            var (report, success) = FeatureSuite.RunAll(path);
            ReturnBuffer(Encoding.UTF8.GetBytes(report), output, length);
            return success ? 0 : 1;
        }
        catch (Exception exception)
        {
            ReturnBuffer(Encoding.UTF8.GetBytes(exception.ToString()), output, length);
            return 1;
        }
    }

    private static void ReturnBuffer(byte[] data, byte** output, int* length)
    {
        var buffer = (byte*)NativeMemory.Alloc((nuint)data.Length);
        data.CopyTo(new Span<byte>(buffer, data.Length));
        *output = buffer;
        *length = data.Length;
    }

    [UnmanagedCallersOnly(EntryPoint = "questpdf_free", CallConvs = [typeof(CallConvCdecl)])]
    public static void Free(void* buffer) => NativeMemory.Free(buffer);
}
