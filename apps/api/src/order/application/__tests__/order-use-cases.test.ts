import type { IOrderRepository } from "../../domain/order.repository";
import { describe, it, expect, mock } from "bun:test";
import { CreateOrderUseCase } from "../create-order.usecase";
import { UpdateOrderStatusUseCase } from "../update-order-status.usecase";
import {
  UnauthorizedOrderActionError,
  InvalidOrderStateTransitionError,
  SelfOrderNotAllowedError,
  OfferNotActiveError,
  DuplicateOrderError,
} from "../../domain/errors";
import type { Order } from "../../domain/order.schema";

describe("Order Use Cases", () => {
  describe("CreateOrderUseCase", () => {
    it("should create an order successfully", async () => {
      const mockOrderRepo = {
        createOrder: mock(
          async (data) =>
            ({ ...data, status: "PENDING", createdAt: new Date(), updatedAt: new Date() }) as Order,
        ),
        getOrderById: mock(),
        getOrdersByRequester: mock(),
        getOrdersByOfferOwner: mock(),
        updateOrderStatus: mock(),
        findPendingByRequesterAndOffer: mock(async () => null),
      };
      const mockOfferService = {
        getOfferPriceAndOwnerAndStatus: mock(async () => ({
          price: 1000,
          ownerId: "owner-123",
          status: "ACTIVE",
        })),
      };

      const useCase = new CreateOrderUseCase(mockOrderRepo, mockOfferService);
      const order = await useCase.execute({
        offerId: "offer-1",
        requesterId: "req-1",
        quantity: 2,
        message: "hi",
      });

      expect(order.price).toBe(1000);
      expect(order.quantity).toBe(2);
      expect(order.status).toBe("PENDING");
      expect(mockOrderRepo.createOrder.mock.calls.length).toBe(1);
    });

    it("should throw OfferNotActiveError if offer is not ACTIVE", async () => {
      const mockOrderRepo = {} as unknown as IOrderRepository;
      const mockOfferService = {
        getOfferPriceAndOwnerAndStatus: mock(async () => ({
          price: 1000,
          ownerId: "owner-123",
          status: "PAUSED",
        })),
      };

      const useCase = new CreateOrderUseCase(mockOrderRepo, mockOfferService);
      await expect(
        useCase.execute({ offerId: "offer-1", requesterId: "req-1", quantity: 1 }),
      ).rejects.toThrow(OfferNotActiveError);
    });

    it("should throw SelfOrderNotAllowedError if requester is owner", async () => {
      const mockOrderRepo = {} as unknown as IOrderRepository;
      const mockOfferService = {
        getOfferPriceAndOwnerAndStatus: mock(async () => ({
          price: 1000,
          ownerId: "req-1",
          status: "ACTIVE",
        })),
      };

      const useCase = new CreateOrderUseCase(mockOrderRepo, mockOfferService);
      await expect(
        useCase.execute({ offerId: "offer-1", requesterId: "req-1", quantity: 1 }),
      ).rejects.toThrow(SelfOrderNotAllowedError);
    });

    it("should throw DuplicateOrderError if a PENDING order already exists", async () => {
      const existingOrder = {
        id: "existing-order",
        offerId: "offer-1",
        requesterId: "req-1",
        status: "PENDING",
      } as Order;
      const mockOrderRepo = {
        createOrder: mock(),
        getOrderById: mock(),
        getOrdersByRequester: mock(),
        getOrdersByOfferOwner: mock(),
        updateOrderStatus: mock(),
        findPendingByRequesterAndOffer: mock(async () => existingOrder),
      };
      const mockOfferService = {
        getOfferPriceAndOwnerAndStatus: mock(async () => ({
          price: 1000,
          ownerId: "owner-123",
          status: "ACTIVE",
        })),
      };

      const useCase = new CreateOrderUseCase(mockOrderRepo, mockOfferService);
      await expect(
        useCase.execute({ offerId: "offer-1", requesterId: "req-1", quantity: 1 }),
      ).rejects.toThrow(DuplicateOrderError);
    });
  });

  describe("UpdateOrderStatusUseCase", () => {
    it("should allow owner to ACCEPT a PENDING order", async () => {
      const mockOrderRepo = {
        getOrderById: mock(
          async () =>
            ({ id: "o1", offerId: "offer-1", requesterId: "req-1", status: "PENDING" }) as Order,
        ),
        updateOrderStatus: mock(async (id, status) => ({ id, status }) as unknown as Order),
        createOrder: mock(),
        getOrdersByRequester: mock(),
        getOrdersByOfferOwner: mock(),
        findPendingByRequesterAndOffer: mock(),
      };
      const mockOfferOwnerService = {
        getOfferOwnerId: mock(async () => "owner-123"),
      };

      const useCase = new UpdateOrderStatusUseCase(mockOrderRepo, mockOfferOwnerService);
      const res = await useCase.execute({
        orderId: "o1",
        accountId: "owner-123",
        newStatus: "ACCEPTED",
      });

      expect(res.status).toBe("ACCEPTED");
    });

    it("should NOT allow requester to ACCEPT order", async () => {
      const mockOrderRepo = {
        getOrderById: mock(
          async () =>
            ({ id: "o1", offerId: "offer-1", requesterId: "req-1", status: "PENDING" }) as Order,
        ),
        updateOrderStatus: mock(),
        createOrder: mock(),
        getOrdersByRequester: mock(),
        getOrdersByOfferOwner: mock(),
        findPendingByRequesterAndOffer: mock(),
      };
      const mockOfferOwnerService = {
        getOfferOwnerId: mock(async () => "owner-123"),
      };

      const useCase = new UpdateOrderStatusUseCase(mockOrderRepo, mockOfferOwnerService);
      await expect(
        useCase.execute({ orderId: "o1", accountId: "req-1", newStatus: "ACCEPTED" }),
      ).rejects.toThrow(InvalidOrderStateTransitionError);
    });

    it("should allow requester to CANCEL a PENDING order", async () => {
      const mockOrderRepo = {
        getOrderById: mock(
          async () =>
            ({ id: "o1", offerId: "offer-1", requesterId: "req-1", status: "PENDING" }) as Order,
        ),
        updateOrderStatus: mock(async (id, status) => ({ id, status }) as unknown as Order),
        createOrder: mock(),
        getOrdersByRequester: mock(),
        getOrdersByOfferOwner: mock(),
        findPendingByRequesterAndOffer: mock(),
      };
      const mockOfferOwnerService = {
        getOfferOwnerId: mock(async () => "owner-123"),
      };

      const useCase = new UpdateOrderStatusUseCase(mockOrderRepo, mockOfferOwnerService);
      const res = await useCase.execute({
        orderId: "o1",
        accountId: "req-1",
        newStatus: "CANCELLED",
      });

      expect(res.status).toBe("CANCELLED");
    });

    it("should throw UnauthorizedOrderActionError if third party attempts update", async () => {
      const mockOrderRepo = {
        getOrderById: mock(
          async () =>
            ({ id: "o1", offerId: "offer-1", requesterId: "req-1", status: "PENDING" }) as Order,
        ),
        updateOrderStatus: mock(),
        createOrder: mock(),
        getOrdersByRequester: mock(),
        getOrdersByOfferOwner: mock(),
        findPendingByRequesterAndOffer: mock(),
      };
      const mockOfferOwnerService = {
        getOfferOwnerId: mock(async () => "owner-123"),
      };

      const useCase = new UpdateOrderStatusUseCase(mockOrderRepo, mockOfferOwnerService);
      await expect(
        useCase.execute({ orderId: "o1", accountId: "third-party", newStatus: "CANCELLED" }),
      ).rejects.toThrow(UnauthorizedOrderActionError);
    });
  });
});
