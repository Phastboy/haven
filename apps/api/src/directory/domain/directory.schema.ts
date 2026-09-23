export interface DirectoryUser {
  id: string;
  username: string | null;
  name: string | null;
  profilePictureUrl: string | null;
}

export interface DirectoryOffer {
  id: string;
  userId: string;
  title: string;
  description: string | null;
  price: number;
  offerType: string;
  images: string[] | null;
  createdAt: Date;
}
