using Microsoft.AspNetCore.Identity;
using Microsoft.EntityFrameworkCore;
using SWEeM.Application.Contracts.Services;
using SWEeM.Application.Dtos;
using SWEeM.Application.Dtos.Project;
using SWEeM.Application.Mappers;
using SWEeM.Domain.Entities;
using SWEeM.Infrastructure.Persistence;

namespace SWEeM.Application.Services;

public class ProjectService(AppDbContext dbContext) : IProjectService
{
    public async Task<Guid> CreateAsync(CreateProjectDto dto, CancellationToken cancellationToken = default)
    {
        var project = dto.ToProject();
        
        dbContext.Projects.Add(project);
        await dbContext.SaveChangesAsync(cancellationToken);
        return project.Id;
    }

    public async Task<PaginatedResult<ProjectDto>> GetAllAsync(
        int page = 1,
        int pageSize = 10,
        CancellationToken cancellationToken = default)
    {
        var query = dbContext.Projects.AsNoTracking();
        var totalCount = await query.CountAsync(cancellationToken);

        var projects = await query
            .Skip((page - 1) * pageSize)
            .Take(pageSize)
            .ToListAsync(cancellationToken);

        var dtos = projects.Select(c => c.ToDto()!).ToList();
        return new PaginatedResult<ProjectDto>(dtos, page, pageSize, totalCount);
    }

    public async Task<ProjectDto?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
    {
        var project = await dbContext.Projects.FindAsync(id, cancellationToken);
        return project?.ToDto();
    }

    public async Task<ProjectDto?> UpdateAsync(Guid id, UpdateProjectDto dto, CancellationToken cancellationToken = default)
    {
        var project = await dbContext.Projects.FindAsync(id, cancellationToken);

        if (project is null)
        {
            return null;
        }

        project.UpdateFrom(dto);

        await dbContext.SaveChangesAsync(cancellationToken);
        return project.ToDto();
    }

    public async Task<bool> DeleteAsync(Guid id, CancellationToken cancellationToken = default)
    {
        var project = await dbContext.Projects.FindAsync(id, cancellationToken);

        if (project is null)
        {
            return false;
        }

        dbContext.Projects.Remove(project);
        await dbContext.SaveChangesAsync(cancellationToken);
        return true;
    }
}