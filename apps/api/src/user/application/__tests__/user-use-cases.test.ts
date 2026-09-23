import type { IUserRepository } from "../../domain/user.repository";
import { expect, test, describe, mock, beforeEach } from "bun:test";
import { CreateUserUseCase } from "../create-user.usecase";
import { UpdateUserUseCase } from "../update-user.usecase";
import { GetUserUseCase } from "../get-user.usecase";
import { ListUsersUseCase } from "../list-users.usecase";
import { DeleteUserUseCase } from "../delete-user.usecase";
import { ConflictError, NotFoundError } from "../../../shared/errors";

describe("User Use Cases", () => {
  const mockUserRepo = {
    findByAccountId: mock(),
    findById: mock(),
    findAll: mock(),
    create: mock(),
    update: mock(),
    delete: mock(),
  };

  beforeEach(() => {
    mockUserRepo.findByAccountId.mockClear();
    mockUserRepo.findById.mockClear();
    mockUserRepo.findAll.mockClear();
    mockUserRepo.create.mockClear();
    mockUserRepo.update.mockClear();
    mockUserRepo.delete.mockClear();
  });

  describe("CreateUserUseCase", () => {
    const useCase = new CreateUserUseCase(mockUserRepo as unknown as IUserRepository);

    test("should throw ConflictError if profile already exists for accountId", async () => {
      mockUserRepo.findByAccountId.mockResolvedValueOnce({ id: "existing-id" });
      await expect(useCase.execute({ accountId: "acc-1" })).rejects.toThrow(ConflictError);
    });

    test("should create and return user if profile does not exist", async () => {
      mockUserRepo.findByAccountId.mockResolvedValueOnce(null);
      mockUserRepo.create.mockResolvedValueOnce({ id: "user-1", accountId: "acc-1" });

      const result = await useCase.execute({ accountId: "acc-1" });
      expect(result.id).toBe("user-1");
      expect(mockUserRepo.create).toHaveBeenCalledWith({ accountId: "acc-1" });
    });
  });

  describe("UpdateUserUseCase", () => {
    const useCase = new UpdateUserUseCase(mockUserRepo as unknown as IUserRepository);

    test("should throw NotFoundError if user not found by accountId", async () => {
      mockUserRepo.findByAccountId.mockResolvedValueOnce(null);
      await expect(useCase.execute("acc-1", { name: "New Name" })).rejects.toThrow(NotFoundError);
    });

    test("should update user if found", async () => {
      mockUserRepo.findByAccountId.mockResolvedValueOnce({ id: "user-1", accountId: "acc-1" });
      mockUserRepo.update.mockResolvedValueOnce({ id: "user-1", name: "New Name" });

      const result = await useCase.execute("acc-1", { name: "New Name" });
      expect(result.name).toBe("New Name");
      expect(mockUserRepo.update).toHaveBeenCalledWith("user-1", { name: "New Name" });
    });
  });

  describe("GetUserUseCase", () => {
    const useCase = new GetUserUseCase(mockUserRepo as unknown as IUserRepository);

    test("should throw NotFoundError if user not found by id", async () => {
      mockUserRepo.findById.mockResolvedValueOnce(null);
      await expect(useCase.execute("user-not-found")).rejects.toThrow(NotFoundError);
    });

    test("should return user if found", async () => {
      mockUserRepo.findById.mockResolvedValueOnce({ id: "user-1" });
      const result = await useCase.execute("user-1");
      expect(result.id).toBe("user-1");
    });
  });

  describe("ListUsersUseCase", () => {
    const useCase = new ListUsersUseCase(mockUserRepo as unknown as IUserRepository);

    test("should return array of users", async () => {
      mockUserRepo.findAll.mockResolvedValueOnce([{ id: "user-1" }]);
      const result = await useCase.execute();
      expect(result.length).toBe(1);
    });
  });

  describe("DeleteUserUseCase", () => {
    const useCase = new DeleteUserUseCase(mockUserRepo as unknown as IUserRepository);

    test("should delete user by id", async () => {
      mockUserRepo.findById.mockResolvedValueOnce({ id: "user-1" });
      mockUserRepo.delete.mockResolvedValueOnce(undefined);
      await useCase.execute("user-1");
      expect(mockUserRepo.delete).toHaveBeenCalledWith("user-1");
    });
  });
});
