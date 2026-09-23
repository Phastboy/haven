import { describe, it, expect, mock, beforeEach } from "bun:test";
import { CreateOfferUseCase } from "../create-offer.usecase";
import { UpdateOfferUseCase } from "../update-offer.usecase";
import { DeleteOfferUseCase } from "../delete-offer.usecase";
import { GetOfferUseCase } from "../get-offer.usecase";
import { ListUserOffersUseCase } from "../list-user-offers.usecase";
import type { IOfferRepository } from "../../domain/offer.repository";
import { OfferNotFoundError, UnauthorizedOfferActionError } from "../../domain/errors";
import type { Offer } from "../../domain/offer.schema";

const mockOffer: Offer = {
  id: "offer-1",
  userId: "user-1",
  title: "Test Offer",
  description: "Test Desc",
  price: 100,
  status: "ACTIVE",
  offerType: "PRODUCT",
  images: null,
  createdAt: new Date(),
  updatedAt: new Date(),
};

class MockOfferRepository implements IOfferRepository {
  create = mock(async () => mockOffer);
  findById = mock(async (id: string) => (id === "offer-1" ? mockOffer : null));
  findByUserId = mock(async () => [mockOffer]);
  findActiveByUserId = mock(async () => [mockOffer]);
  update = mock(async () => mockOffer);
  delete = mock(async () => true);
}

describe("Offer Use Cases", () => {
  let repository: MockOfferRepository;

  beforeEach(() => {
    repository = new MockOfferRepository();
  });

  describe("CreateOfferUseCase", () => {
    it("should create an offer", async () => {
      const useCase = new CreateOfferUseCase(repository);
      const result = await useCase.execute("user-1", { title: "Test Offer" });
      expect(result).toEqual(mockOffer);
      expect(repository.create).toHaveBeenCalledWith("user-1", { title: "Test Offer" });
    });
  });

  describe("UpdateOfferUseCase", () => {
    it("should throw OfferNotFoundError if not found", async () => {
      const useCase = new UpdateOfferUseCase(repository);
      expect(useCase.execute("user-1", "not-found", {})).rejects.toThrow(OfferNotFoundError);
    });

    it("should throw UnauthorizedOfferActionError if wrong user", async () => {
      const useCase = new UpdateOfferUseCase(repository);
      expect(useCase.execute("user-2", "offer-1", {})).rejects.toThrow(
        UnauthorizedOfferActionError,
      );
    });

    it("should update offer successfully", async () => {
      const useCase = new UpdateOfferUseCase(repository);
      const result = await useCase.execute("user-1", "offer-1", { title: "New" });
      expect(result).toEqual(mockOffer);
      expect(repository.update).toHaveBeenCalledWith("offer-1", { title: "New" });
    });
  });

  describe("DeleteOfferUseCase", () => {
    it("should throw OfferNotFoundError if not found", async () => {
      const useCase = new DeleteOfferUseCase(repository);
      expect(useCase.execute("user-1", "not-found")).rejects.toThrow(OfferNotFoundError);
    });

    it("should throw UnauthorizedOfferActionError if wrong user", async () => {
      const useCase = new DeleteOfferUseCase(repository);
      expect(useCase.execute("user-2", "offer-1")).rejects.toThrow(UnauthorizedOfferActionError);
    });

    it("should delete offer successfully", async () => {
      const useCase = new DeleteOfferUseCase(repository);
      await useCase.execute("user-1", "offer-1");
      expect(repository.delete).toHaveBeenCalledWith("offer-1");
    });
  });

  describe("GetOfferUseCase", () => {
    it("should throw OfferNotFoundError if not found", async () => {
      const useCase = new GetOfferUseCase(repository);
      expect(useCase.execute("not-found")).rejects.toThrow(OfferNotFoundError);
    });

    it("should return offer successfully", async () => {
      const useCase = new GetOfferUseCase(repository);
      const result = await useCase.execute("offer-1");
      expect(result).toEqual(mockOffer);
    });
  });

  describe("ListUserOffersUseCase", () => {
    it("should call findByUserId when requester is the owner", async () => {
      const useCase = new ListUserOffersUseCase(repository);
      const result = await useCase.execute("user-1", "user-1");
      expect(result.data).toEqual([mockOffer]);
      expect(repository.findByUserId).toHaveBeenCalledWith("user-1");
    });

    it("should call findActiveByUserId for a third-party requester", async () => {
      const useCase = new ListUserOffersUseCase(repository);
      const result = await useCase.execute("user-1", "user-2");
      expect(result.data).toEqual([mockOffer]);
      expect(repository.findActiveByUserId).toHaveBeenCalledWith("user-1");
    });

    it("should call findActiveByUserId for an unauthenticated requester", async () => {
      const useCase = new ListUserOffersUseCase(repository);
      const result = await useCase.execute("user-1");
      expect(result.data).toEqual([mockOffer]);
      expect(repository.findActiveByUserId).toHaveBeenCalledWith("user-1");
    });
  });
});
