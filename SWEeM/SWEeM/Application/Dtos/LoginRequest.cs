namespace SWEeM.Application.Dtos;

public record LoginRequest(
    string Login,
    string Password);