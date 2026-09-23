export const typeDefs = `
  type User {
    id: ID!
    username: String
    name: String
    profilePictureUrl: String
  }

  type Offer {
    id: ID!
    title: String!
    description: String
    price: Int!
    offerType: String!
    images: [String!]
    createdAt: String!
    user: User
  }

  type Query {
    activeOffers(limit: Int = 50, offset: Int = 0): [Offer!]!
  }
`;
