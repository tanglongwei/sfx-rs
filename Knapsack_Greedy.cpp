#include<iostream>
using namespace std;
void sort(int w[],int v[],int m[],int n)
{
    for(int i=0;i<=n-2;i++)
    for(int j=i+1;j<=n-1;j++)
      {
	if(m[j]>m[i]){
	int temp;
	int tempa;
	int tempb;
	temp=m[j];
	tempa=w[j];
	tempb=v[j];
	m[j]=m[i];
	w[j]=w[i];
	v[j]=v[i];
	m[i]=temp;
	w[i]=tempa;
	v[i]=tempb;
	}
      }

    }



int main()
{
  float e;
  int k=0;
  int c=50;
  int n=3;
  int s=0;
  int w[3]={20,30,10};
  int v[3]={60,120,50};
  int x[3]={0,0,0};
  int m[n];
  // for(int j=0;j<n;j++)
  //   {
  //     m[j]=v[j]/w[j];
  //     cout<<m[j]<<" ";
  //   }
  // cout<<endl;
  // sort(w,v,m,3);

  // for(int j=0;j<n;j++)
  //   {
  //         m[j]=v[j]/w[j];
  //     cout<<v[j]<<" ";
  //   }
  // cout<<endl;
  
   while(s<c && k<=n-1)
    {
      cout<<"w"<<k<<":"<<w[k]<<endl;
      s=s+w[k];
      k++;			
    }
  cout<<"k="<<k<<endl;
  cout<<"s="<<s<<endl;
  // cout<<"sum"<<s<<endl;
  if(k==n+1)
    {
  for(int j=0;j<=n-1;j++)
    {
      x[j]=1;
      cout<<x[j]<<"  ";
    }
  return 0;
    }
  else
    {
      for(int j=0;j<=k-2;j++)
	{
	  x[j]=1;
	}
      float e=(c-(s-w[k-1]))/(float)w[k-1];//x[k-1]
    cout<<"e="<<e<<endl;
    for(int j=0;j<=k-2;j++)
      {
    cout<<x[j]<<" ";
      }
    cout<<e<<"  ";
    for(int j=k;j<=n-1;j++)
      {
	  cout<<x[j]<<" ";
      }
    
  return 0;
    }
}
