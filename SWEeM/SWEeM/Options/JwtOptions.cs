namespace SWEeM.Options;

public class JwtOptions
{
    public const string SectionName = "Jwt";

    public string Key { get; set; } = string.Empty;
    public string Issuer { get; set; } = string.Empty;
    public string Audience { get; set; } = string.Empty;

    public void Validate()
    {
        if (string.IsNullOrWhiteSpace(Key))
            throw new InvalidOperationException($"JWT configuration '{SectionName}:Key' is required.");

        if (Key.Length < 32)
            throw new InvalidOperationException("JWT Key must be at least 32 characters long.");

        if (string.IsNullOrWhiteSpace(Issuer))
            throw configError("Issuer");

        if (string.IsNullOrWhiteSpace(Audience))
            throw configError("Audience");
        return;

        InvalidOperationException configError(string name) =>
            new($"JWT configuration '{SectionName}:{name}' is required.");
    }
}