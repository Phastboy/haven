import type { Mock } from "bun:test";
import type { IFulfillmentRepository } from "../../domain/fulfillment.repository";
import { describe, it, expect, mock } from "bun:test";
import { DeliverFulfillmentUseCase } from "../deliver-fulfillment.usecase";
import { AcceptFulfillmentUseCase } from "../accept-fulfillment.usecase";
import { RequestRevisionUseCase } from "../request-revision.usecase";
import { AutoCompleteExpiredUseCase } from "../auto-complete-expired.usecase";
import {
  InvalidFulfillmentStateTransitionError,
  RevisionNotApplicableError,
  UnauthorizedFulfillmentActionError,
} from "../../domain/errors";

describe("Fulfillment Use Cases", () => {
  describe("DeliverFulfillmentUseCase", () => {
    it("should throw if order is not ACCEPTED", async () => {
      const repo = {} as unknown as IFulfillmentRepository;
      const service = {
        getOrderDetails: mock(async () => ({
          id: "o1",
          status: "PENDING",
          requesterId: "req1",
          offerId: "off1",
        })),
        getOfferTypeAndOwner: mock(),
        updateOrderStatus: mock(),
      };
      const usecase = new DeliverFulfillmentUseCase(repo, service);
      await expect(
        usecase.execute({ orderId: "o1", accountId: "owner1", autoReviewDays: 3 }),
      ).rejects.toThrow(InvalidFulfillmentStateTransitionError);
    });

    it("should throw if user is not the owner", async () => {
      const repo = {} as unknown as IFulfillmentRepository;
      const service = {
        getOrderDetails: mock(async () => ({
          id: "o1",
          status: "ACCEPTED",
          offerId: "off1",
          requesterId: "req1",
        })),
        getOfferTypeAndOwner: mock(async () => ({ offerType: "SERVICE", ownerId: "owner1" })),
        updateOrderStatus: mock(),
      };
      const usecase = new DeliverFulfillmentUseCase(repo, service);
      await expect(
        usecase.execute({ orderId: "o1", accountId: "hacker", autoReviewDays: 3 }),
      ).rejects.toThrow(UnauthorizedFulfillmentActionError);
    });

    it("should successfully deliver an accepted order", async () => {
      const repo = {
        getFulfillmentByOrderId: mock(async () => null),
        createFulfillment: mock(async (data: { id: string; orderId: string }) => ({
          ...data,
          status: "PENDING",
        })),
        updateFulfillmentStatus: mock(async (id: string, status: string) => ({ id, status })),
      } as unknown as IFulfillmentRepository;
      const service = {
        getOrderDetails: mock(async () => ({
          id: "o1",
          status: "ACCEPTED",
          offerId: "off1",
          requesterId: "req1",
        })),
        getOfferTypeAndOwner: mock(async () => ({ offerType: "SERVICE", ownerId: "owner1" })),
        updateOrderStatus: mock(),
      };
      const usecase = new DeliverFulfillmentUseCase(repo, service);
      const result = await usecase.execute({
        orderId: "o1",
        accountId: "owner1",
        autoReviewDays: 3,
      });
      expect(result.status).toBe("DELIVERED");
      expect(
        (repo.updateFulfillmentStatus as Mock<(...args: unknown[]) => unknown>).mock.calls.length,
      ).toBe(1);
    });
  });

  describe("AcceptFulfillmentUseCase", () => {
    it("should allow requester to accept DELIVERED fulfillment", async () => {
      const repo = {
        getFulfillmentByOrderId: mock(async () => ({
          id: "f1",
          status: "DELIVERED",
          orderId: "o1",
        })),
        updateFulfillmentStatus: mock(async (id: string, status: string) => ({ id, status })),
      } as unknown as IFulfillmentRepository;
      const service = {
        getOrderDetails: mock(async () => ({
          id: "o1",
          status: "ACCEPTED",
          requesterId: "req1",
          offerId: "off1",
        })),
        getOfferTypeAndOwner: mock(),
        updateOrderStatus: mock(async () => {}),
      };
      const usecase = new AcceptFulfillmentUseCase(repo, service);
      const result = await usecase.execute({ orderId: "o1", accountId: "req1" });
      expect(result.status).toBe("COMPLETED");
      expect(service.updateOrderStatus.mock.calls.length).toBe(1);
    });

    it("should throw if non-requester attempts to accept", async () => {
      const repo = {} as unknown as IFulfillmentRepository;
      const service = {
        getOrderDetails: mock(async () => ({
          id: "o1",
          status: "ACCEPTED",
          requesterId: "req1",
          offerId: "off1",
        })),
        getOfferTypeAndOwner: mock(),
        updateOrderStatus: mock(),
      };
      const usecase = new AcceptFulfillmentUseCase(repo, service);
      await expect(usecase.execute({ orderId: "o1", accountId: "owner1" })).rejects.toThrow(
        UnauthorizedFulfillmentActionError,
      );
    });
  });

  describe("RequestRevisionUseCase", () => {
    it("should throw RevisionNotApplicableError if not a SERVICE", async () => {
      const repo = {} as unknown as IFulfillmentRepository;
      const service = {
        getOrderDetails: mock(async () => ({
          id: "o1",
          status: "ACCEPTED",
          requesterId: "req1",
          offerId: "off1",
        })),
        getOfferTypeAndOwner: mock(async () => ({ offerType: "PRODUCT", ownerId: "owner1" })),
        updateOrderStatus: mock(),
      };
      const usecase = new RequestRevisionUseCase(repo, service);
      await expect(
        usecase.execute({ orderId: "o1", accountId: "req1", reason: "broken" }),
      ).rejects.toThrow(RevisionNotApplicableError);
    });
  });

  describe("AutoCompleteExpiredUseCase", () => {
    it("should auto complete all expired fulfillments", async () => {
      const repo = {
        getExpiredFulfillments: mock(async () => [
          { id: "f1", orderId: "o1", status: "DELIVERED" },
          { id: "f2", orderId: "o2", status: "DELIVERED" },
        ]),
        updateFulfillmentStatus: mock(async () => {}),
      } as unknown as IFulfillmentRepository;
      const service = {
        getOrderDetails: mock(),
        getOfferTypeAndOwner: mock(),
        updateOrderStatus: mock(async () => {}),
      };
      const usecase = new AutoCompleteExpiredUseCase(repo, service);
      const count = await usecase.execute();

      expect(count).toBe(2);
      expect(
        (repo.updateFulfillmentStatus as Mock<(...args: unknown[]) => unknown>).mock.calls.length,
      ).toBe(2);
      expect(service.updateOrderStatus.mock.calls.length).toBe(2);
    });
  });
});
