using Microsoft.AspNetCore.Identity;
using Microsoft.EntityFrameworkCore;
using SWEeM.Application.Contracts.Services;
using SWEeM.Application.Dtos;
using SWEeM.Application.Dtos.User;
using SWEeM.Application.Mappers;
using SWEeM.Domain.Entities;
using SWEeM.Infrastructure.Persistence;

namespace SWEeM.Application.Services;

public class UserService(AppDbContext dbContext, IPasswordHasher<User> hasher) : IUserService
{
    public async Task<Guid> CreateAsync(CreateUserDto dto, CancellationToken cancellationToken = default)
    {
        var passwordHash = hasher.HashPassword(null, dto.Password);

        var user = dto.ToUser(passwordHash);

        dbContext.Users.Add(user);
        await dbContext.SaveChangesAsync(cancellationToken);
        return user.Id;
    }

    public async Task<PaginatedResult<UserDto>> GetAllAsync(
        int page = 1,
        int pageSize = 10,
        CancellationToken cancellationToken = default)
    {
        var query = dbContext.Users.AsNoTracking();
        var totalCount = await query.CountAsync(cancellationToken);

        var users = await query
            .Skip((page - 1) * pageSize)
            .Take(pageSize)
            .ToListAsync(cancellationToken);

        var dtos = users.Select(c => c.ToDto()!).ToList();
        return new PaginatedResult<UserDto>(dtos, page, pageSize, totalCount);
    }

    public async Task<UserDto?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
    {
        var user =  await dbContext.Users.FindAsync(id, cancellationToken);
        return user?.ToDto();
    }

    public async Task<UserDto?> UpdateAsync(Guid id, UpdateUserDto dto, CancellationToken cancellationToken = default)
    {
        var user = await dbContext.Users.FindAsync(id, cancellationToken);

        if (user is null)
        {
            return null;
        }

        user.UpdateFrom(dto);

        await dbContext.SaveChangesAsync(cancellationToken);
        return user.ToDto();
    }

    public async Task<UserDto?> ValidateCredentialsAsync(
        string login,
        string password,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(login) || string.IsNullOrWhiteSpace(password))
            return null;

        // Find user by login (case-sensitive or insensitive as needed)
        var user = await dbContext.Users
            .AsNoTracking()
            .FirstOrDefaultAsync(u => u.Login == login, cancellationToken);

        if (user == null)
            return null;

        // Verify password
        var result = hasher.VerifyHashedPassword(null, user.PasswordHash, password);

        if (result == PasswordVerificationResult.Success)
        {
            return new UserDto(user.Id, user.Name, user.Login, user.Role);
        }

        return null;
    }

    public async Task<bool> DeleteAsync(Guid id, CancellationToken cancellationToken = default)
    {
        var user = await dbContext.Users.FindAsync(id, cancellationToken);

        if (user is null)
        {
            return false;
        }

        dbContext.Users.Remove(user);
        await dbContext.SaveChangesAsync(cancellationToken);
        return true;
    }
}