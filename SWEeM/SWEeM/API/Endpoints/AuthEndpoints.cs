using Microsoft.Extensions.Options;
using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Text;
using Microsoft.IdentityModel.Tokens;
using SWEeM.Application.Dtos;
using SWEeM.Application.Services;
using SWEeM.Domain.Enums;
using SWEeM.Options;

namespace SWEeM.API.Endpoints;

public static class AuthEndpoints
{
    public static RouteGroupBuilder MapAuthEndpoints(this WebApplication app)
    {
        var group = app.MapGroup("/auth").WithTags("Auth");

        group.MapPost("/login", async (
            LoginRequest request,
            UserService userService,
            IOptions<JwtOptions> jwtOptions,
            CancellationToken ct) =>
        {
            var user = await userService.ValidateCredentialsAsync(request.Login, request.Password, ct);
            if (user is null)
                return Results.Unauthorized();

            var options = jwtOptions.Value;
            var token = GenerateJwtToken(user.Id, user.Role, options.Key, options.Issuer, options.Audience);

            return Results.Ok(new { Token = token });
        });

        return group;
    }

    private static string GenerateJwtToken(Guid userId, Role role, string key, string issuer, string audience)
    {
        var claims = new[]
        {
            new Claim(ClaimTypes.NameIdentifier, userId.ToString()),
            new Claim(ClaimTypes.Role, role.ToString())
        };

        var securityKey = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(key));
        var credentials = new SigningCredentials(securityKey, SecurityAlgorithms.HmacSha256);

        var token = new JwtSecurityToken(
            issuer: issuer,
            audience: audience,
            claims: claims,
            expires: DateTime.UtcNow.AddHours(1),
            signingCredentials: credentials);

        return new JwtSecurityTokenHandler().WriteToken(token);
    }
}