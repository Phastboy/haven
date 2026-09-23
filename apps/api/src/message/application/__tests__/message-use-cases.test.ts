import { describe, it, expect, mock, beforeEach } from "bun:test";
import { CreateThreadUseCase } from "../create-thread.usecase";
import { SendMessageUseCase } from "../send-message.usecase";
import { GetThreadsUseCase } from "../get-threads.usecase";
import { GetMessagesUseCase } from "../get-messages.usecase";
import type { SqlMessageRepository } from "../../infrastructure/sql-message.repository";
import { ThreadNotFoundError, UnauthorizedThreadAccessError } from "../../domain/errors";

describe("Message Use Cases", () => {
  let mockRepo: Record<keyof SqlMessageRepository, ReturnType<typeof mock>>;

  beforeEach(() => {
    mockRepo = {
      findThreadById: mock(),
      findThreadByParticipants: mock(),
      createThread: mock(),
      getUserThreads: mock(),
      getThreadMessages: mock(),
      sendMessage: mock(),
      markMessagesAsRead: mock(),
    };
  });

  describe("CreateThreadUseCase", () => {
    it("should prevent creating a thread with oneself", async () => {
      const useCase = new CreateThreadUseCase(mockRepo as unknown as SqlMessageRepository);
      expect(useCase.execute("user1", "user1")).rejects.toThrow(
        "Cannot create a thread with yourself.",
      );
    });

    it("should return existing thread if it exists", async () => {
      mockRepo.findThreadByParticipants.mockResolvedValue({ id: "thread1" } as unknown);
      const useCase = new CreateThreadUseCase(mockRepo as unknown as SqlMessageRepository);

      const thread = await useCase.execute("user1", "user2");
      expect(thread.id).toBe("thread1");
      expect(mockRepo.createThread).not.toHaveBeenCalled();
    });

    it("should create new thread if none exists", async () => {
      mockRepo.findThreadByParticipants.mockResolvedValue(null);
      mockRepo.createThread.mockResolvedValue({ id: "thread2" } as unknown);
      const useCase = new CreateThreadUseCase(mockRepo as unknown as SqlMessageRepository);

      const thread = await useCase.execute("user1", "user2");
      expect(thread.id).toBe("thread2");
      expect(mockRepo.createThread).toHaveBeenCalledWith("user1", "user2");
    });
  });

  describe("SendMessageUseCase", () => {
    const mockEventBus = { publish: mock(), subscribe: mock() };

    it("should throw ThreadNotFoundError if thread does not exist", async () => {
      mockRepo.findThreadById.mockResolvedValue(null);
      const useCase = new SendMessageUseCase(
        mockRepo as unknown as SqlMessageRepository,
        mockEventBus,
      );
      expect(useCase.execute("thread1", "user1", "hello")).rejects.toThrow(ThreadNotFoundError);
    });

    it("should throw if sender is not in thread", () => {
      mockRepo.findThreadById.mockResolvedValue({
        id: "thread1",
        participant1Id: "user2",
        participant2Id: "user3",
      } as unknown);
      const useCase = new SendMessageUseCase(
        mockRepo as unknown as SqlMessageRepository,
        mockEventBus,
      );
      expect(useCase.execute("thread1", "user1", "hello")).rejects.toThrow(
        UnauthorizedThreadAccessError,
      );
    });

    it("should send message if authorized", async () => {
      mockRepo.findThreadById.mockResolvedValue({
        id: "thread1",
        participant1Id: "user1",
        participant2Id: "user3",
      } as unknown);
      mockRepo.sendMessage.mockResolvedValue({ id: "msg1" } as unknown);
      const useCase = new SendMessageUseCase(
        mockRepo as unknown as SqlMessageRepository,
        mockEventBus,
      );

      const msg = await useCase.execute("thread1", "user1", "hello");
      expect(msg.id).toBe("msg1");
      expect(mockRepo.sendMessage).toHaveBeenCalledWith("thread1", "user1", "hello");
    });
  });

  describe("GetThreadsUseCase", () => {
    it("should get user threads", async () => {
      mockRepo.getUserThreads.mockResolvedValue([{ id: "thread1" }] as unknown);
      const useCase = new GetThreadsUseCase(mockRepo as unknown as SqlMessageRepository);
      const threads = await useCase.execute("user1");
      expect(threads).toHaveLength(1);
    });
  });

  describe("GetMessagesUseCase", () => {
    it("should throw if thread not found", () => {
      mockRepo.findThreadById.mockResolvedValue(null);
      const useCase = new GetMessagesUseCase(mockRepo as unknown as SqlMessageRepository);
      expect(useCase.execute("thread1", "user1")).rejects.toThrow(ThreadNotFoundError);
    });

    it("should throw if user not in thread", () => {
      mockRepo.findThreadById.mockResolvedValue({
        id: "thread1",
        participant1Id: "user2",
        participant2Id: "user3",
      } as unknown);
      const useCase = new GetMessagesUseCase(mockRepo as unknown as SqlMessageRepository);
      expect(useCase.execute("thread1", "user1")).rejects.toThrow(UnauthorizedThreadAccessError);
    });

    it("should get messages", async () => {
      mockRepo.findThreadById.mockResolvedValue({
        id: "thread1",
        participant1Id: "user1",
        participant2Id: "user3",
      } as unknown);
      mockRepo.getThreadMessages.mockResolvedValue([{ id: "msg1" }] as unknown);
      const useCase = new GetMessagesUseCase(mockRepo as unknown as SqlMessageRepository);
      const messages = await useCase.execute("thread1", "user1");
      expect(messages).toHaveLength(1);
    });
  });
});
